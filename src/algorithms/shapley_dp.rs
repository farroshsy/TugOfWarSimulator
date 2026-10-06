use crate::game::{GameState, Team};
use crate::strategies::{Strategy, TeamAction, allocation_to_action, Allocation};
use crate::algorithms::{generate_allocations, solve_matrix_game, MatrixGameSolution};
use crate::algorithms::dp::StateKey;
use crate::game::simulation::{step_outcomes, calculate_score};
use rand::{Rng, SeedableRng};
use std::collections::HashMap;
use std::sync::Arc;

/// Simultaneous-move DP (Shapley, 1953).
///
/// Both teams choose allocations at the same time, so each state is a
/// matrix game: M[a][b] = V(next state after A plays a and B plays b).
/// Instead of the sequential `max_a min_b`, the state value is the value of
/// that matrix game, which may require mixed (randomized) strategies.
/// See docs/research-solution-methods.md, section 2.
pub struct ShapleyDpSolver {
    objective: Objective,
    cache: HashMap<StateKey, MatrixGameSolution>,
    // Arc: the recursive solve needs &mut self, so the loops hold cheap
    // shared handles instead of cloning the allocation lists at every state.
    allocations_a: Arc<[Allocation]>,
    allocations_b: Arc<[Allocation]>,
}

/// What Team A's terminal payoff is (Team B gets the negative).
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Objective {
    /// Final score differential A - B. Maximizes the expected margin.
    ScoreDifferential,
    /// +1 win, 0 draw, -1 loss. Maximizes P(win) - P(loss).
    WinProbability,
}

impl Objective {
    fn terminal_value(&self, state: &GameState) -> f64 {
        let score = calculate_score(state);
        let diff = (score.team_a - score.team_b) as f64;
        match self {
            Objective::ScoreDifferential => diff,
            // Not f64::signum: it maps 0.0 to 1.0, which would count draws as wins.
            Objective::WinProbability => {
                if diff > 0.0 {
                    1.0
                } else if diff < 0.0 {
                    -1.0
                } else {
                    0.0
                }
            }
        }
    }
}

impl ShapleyDpSolver {
    pub fn new(state: &GameState) -> Self {
        Self::with_objective(state, Objective::ScoreDifferential)
    }

    pub fn with_objective(state: &GameState, objective: Objective) -> Self {
        let ropes = state.ropes.len();
        Self {
            objective,
            cache: HashMap::new(),
            allocations_a: generate_allocations(state.get_team_player_count(Team::A), ropes).into(),
            allocations_b: generate_allocations(state.get_team_player_count(Team::B), ropes).into(),
        }
    }

    /// Game value for Team A under optimal mixed play, in units of the objective.
    pub fn solve(&mut self, state: &GameState) -> f64 {
        self.stage(state).value
    }

    /// Value and optimal mixed strategies for both teams at this state.
    /// Strategies are indexed like `allocations(team)`; empty at terminal states.
    pub fn stage(&mut self, state: &GameState) -> &MatrixGameSolution {
        let key = StateKey::from_state(state);
        if !self.cache.contains_key(&key) {
            let solution = self.compute(state);
            self.cache.insert(key.clone(), solution);
        }
        &self.cache[&key]
    }

    fn compute(&mut self, state: &GameState) -> MatrixGameSolution {
        if state.finished() {
            return MatrixGameSolution {
                value: self.objective.terminal_value(state),
                row_strategy: vec![],
                col_strategy: vec![],
            };
        }

        let allocations_a = Arc::clone(&self.allocations_a);
        let allocations_b = Arc::clone(&self.allocations_b);
        let mut payoff = vec![vec![0.0; allocations_b.len()]; allocations_a.len()];

        for (i, allocation_a) in allocations_a.iter().enumerate() {
            let action_a = allocation_to_action(state, Team::A, &allocation_a.counts);
            for (j, allocation_b) in allocations_b.iter().enumerate() {
                let action_b = allocation_to_action(state, Team::B, &allocation_b.counts);
                // Expected value over chance outcomes (one outcome if deterministic).
                payoff[i][j] = step_outcomes(state, &action_a, &action_b)
                    .into_iter()
                    .map(|(p, next_state)| p * self.solve(&next_state))
                    .sum();
            }
        }

        solve_matrix_game(&payoff)
    }

    pub fn allocations(&self, team: Team) -> &[Allocation] {
        match team {
            Team::A => &self.allocations_a,
            Team::B => &self.allocations_b,
        }
    }

    pub fn cache_size(&self) -> usize {
        self.cache.len()
    }
}

/// Plays the Shapley DP mixed strategy for either team, sampling an
/// allocation each turn with a seeded RNG.
pub struct ShapleyDpStrategy {
    objective: Objective,
    solver: Option<ShapleyDpSolver>,
    rng: rand::rngs::StdRng,
}

impl ShapleyDpStrategy {
    pub fn new(seed: u64) -> Self {
        Self::with_objective(seed, Objective::ScoreDifferential)
    }

    pub fn with_objective(seed: u64, objective: Objective) -> Self {
        Self {
            objective,
            solver: None,
            rng: rand::rngs::StdRng::seed_from_u64(seed),
        }
    }
}

impl Strategy for ShapleyDpStrategy {
    fn decide(&mut self, state: &GameState, team: Team) -> TeamAction {
        // Solver is built on first use: it needs the game's team sizes.
        let objective = self.objective;
        let solver = self.solver.get_or_insert_with(|| ShapleyDpSolver::with_objective(state, objective));
        let stage = solver.stage(state);
        let probabilities = match team {
            Team::A => stage.row_strategy.clone(),
            Team::B => stage.col_strategy.clone(),
        };
        if probabilities.is_empty() {
            return TeamAction::empty();
        }

        let r: f64 = self.rng.gen();
        let mut cumulative = 0.0;
        let mut choice = probabilities.len() - 1;
        for (i, p) in probabilities.iter().enumerate() {
            cumulative += p;
            if r < cumulative {
                choice = i;
                break;
            }
        }

        let allocation = &solver.allocations(team)[choice];
        allocation_to_action(state, team, &allocation.counts)
    }

    fn name(&self) -> &str {
        match self.objective {
            Objective::ScoreDifferential => "ShapleyDP",
            Objective::WinProbability => "ShapleyDP-Win",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::algorithms::DpSolver;
    use crate::game::create_small_game;

    fn tiny_game(duration: usize) -> GameState {
        let mut state = create_small_game();
        state.rules.duration = duration;
        state
    }

    #[test]
    fn terminal_state_value_is_score_differential() {
        let state = tiny_game(0);
        let mut solver = ShapleyDpSolver::new(&state);
        assert_eq!(solver.solve(&state), 0.0);
    }

    #[test]
    fn shapley_value_is_at_least_sequential_maximin() {
        // Pure maximin lets B react to A's move, so it can only undervalue A.
        let mut state = tiny_game(4);
        // Make teams unequal so the game is not trivially symmetric.
        for p in state.players.iter_mut().filter(|p| p.team == Team::A) {
            p.strength *= 1.3;
        }
        let mut shapley = ShapleyDpSolver::new(&state);
        let mut maximin = DpSolver::new(3, 2);
        let v_shapley = shapley.solve(&state);
        let v_maximin = maximin.solve(&state);
        assert!(v_shapley >= v_maximin - 1e-9, "{} < {}", v_shapley, v_maximin);
    }

    #[test]
    fn strategies_are_probability_distributions() {
        let state = tiny_game(3);
        let mut solver = ShapleyDpSolver::new(&state);
        let stage = solver.stage(&state).clone();
        for strategy in [&stage.row_strategy, &stage.col_strategy] {
            let sum: f64 = strategy.iter().sum();
            assert!((sum - 1.0).abs() < 1e-6);
            assert!(strategy.iter().all(|&p| p >= -1e-9));
        }
    }

    #[test]
    fn forced_win_implies_score_value_has_same_sign() {
        // If one team can guarantee a win (win-objective value is +/-1), its
        // equilibrium strategy guarantees a strictly positive margin for it
        // in every outcome, so the score-objective value has the same sign.
        use crate::game::create_arena_game;
        use rand::{Rng, SeedableRng};
        let mut forced = 0;
        // Seed 1: A forces a win; seed 2: B forces a win.
        for seed in [1u64, 2] {
            let mut state = create_arena_game();
            let mut rng = rand::rngs::StdRng::seed_from_u64(seed);
            for p in &mut state.players {
                p.strength *= 1.0 + rng.gen_range(-0.2..=0.2);
            }
            let win = ShapleyDpSolver::with_objective(&state, Objective::WinProbability).solve(&state);
            let score = ShapleyDpSolver::new(&state).solve(&state);
            assert!((-1.0..=1.0).contains(&win));
            if (win.abs() - 1.0).abs() < 1e-9 {
                forced += 1;
                assert!(score * win > 0.0, "seed {}: win {} score {}", seed, win, score);
            }
        }
        assert_eq!(forced, 2);
    }

    #[test]
    fn noise_makes_win_probability_fractional() {
        // Deterministic arena: outcome is forced (value exactly -1, 0 or 1).
        // With stochastic physics the stronger team can still lose.
        use crate::game::create_noisy_uneven_arena_game;
        let state = create_noisy_uneven_arena_game();
        let win = ShapleyDpSolver::with_objective(&state, Objective::WinProbability).solve(&state);
        assert!(win > 0.0 && win < 1.0, "{}", win);
    }

    #[test]
    fn strategy_plays_for_both_teams() {
        let state = tiny_game(3);
        let mut strategy = ShapleyDpStrategy::new(1);
        // Must not panic or fall back for Team B.
        strategy.decide(&state, Team::A);
        strategy.decide(&state, Team::B);
    }
}
