use crate::game::{GameState, Team};
use crate::strategies::{Strategy, TeamAction, allocation_to_action, Allocation};
use crate::algorithms::{generate_allocations, solve_matrix_game, MatrixGameSolution};
use crate::algorithms::dp::StateKey;
use crate::game::simulation::{step, calculate_score};
use rand::{Rng, SeedableRng};
use std::collections::HashMap;

/// Simultaneous-move DP (Shapley, 1953).
///
/// Both teams choose allocations at the same time, so each state is a
/// matrix game: M[a][b] = V(next state after A plays a and B plays b).
/// Instead of the sequential `max_a min_b`, the state value is the value of
/// that matrix game, which may require mixed (randomized) strategies.
/// See docs/research-solution-methods.md, section 2.
pub struct ShapleyDpSolver {
    cache: HashMap<StateKey, MatrixGameSolution>,
    allocations_a: Vec<Allocation>,
    allocations_b: Vec<Allocation>,
}

impl ShapleyDpSolver {
    pub fn new(state: &GameState) -> Self {
        let ropes = state.ropes.len();
        Self {
            cache: HashMap::new(),
            allocations_a: generate_allocations(state.get_team_player_count(Team::A), ropes),
            allocations_b: generate_allocations(state.get_team_player_count(Team::B), ropes),
        }
    }

    /// Game value for Team A (score differential A - B) under optimal mixed play.
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
            let score = calculate_score(state);
            return MatrixGameSolution {
                value: (score.team_a - score.team_b) as f64,
                row_strategy: vec![],
                col_strategy: vec![],
            };
        }

        let allocations_a = self.allocations_a.clone();
        let allocations_b = self.allocations_b.clone();
        let mut payoff = vec![vec![0.0; allocations_b.len()]; allocations_a.len()];

        for (i, allocation_a) in allocations_a.iter().enumerate() {
            for (j, allocation_b) in allocations_b.iter().enumerate() {
                let mut next_state = state.clone();
                let action_a = allocation_to_action(&next_state, Team::A, &allocation_a.counts);
                let action_b = allocation_to_action(&next_state, Team::B, &allocation_b.counts);
                step(&mut next_state, &action_a, &action_b);
                payoff[i][j] = self.solve(&next_state);
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
    solver: Option<ShapleyDpSolver>,
    rng: rand::rngs::StdRng,
}

impl ShapleyDpStrategy {
    pub fn new(seed: u64) -> Self {
        Self {
            solver: None,
            rng: rand::rngs::StdRng::seed_from_u64(seed),
        }
    }
}

impl Strategy for ShapleyDpStrategy {
    fn decide(&mut self, state: &GameState, team: Team) -> TeamAction {
        // Solver is built on first use: it needs the game's team sizes.
        let solver = self.solver.get_or_insert_with(|| ShapleyDpSolver::new(state));
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
        "ShapleyDP"
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
    fn strategy_plays_for_both_teams() {
        let state = tiny_game(3);
        let mut strategy = ShapleyDpStrategy::new(1);
        // Must not panic or fall back for Team B.
        strategy.decide(&state, Team::A);
        strategy.decide(&state, Team::B);
    }
}
