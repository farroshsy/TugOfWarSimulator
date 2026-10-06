use crate::game::{GameState, Team};
use crate::strategies::{Strategy, TeamAction, allocation_to_action, Allocation};
use crate::algorithms::generate_allocations;
use crate::game::simulation::{step, calculate_score};
use rand::{Rng, SeedableRng};
use std::collections::HashMap;

/// Simultaneous-move Monte Carlo Tree Search with regret matching
/// (SM-MCTS-RM, Lisý et al. 2013). Approximates the Shapley DP solution on
/// games too large to solve exactly: instead of enumerating every state, it
/// samples paths from the current state and learns a mixed strategy at each
/// visited node. Regret matching is Hannan-consistent, so the root's
/// *average* strategy converges to an approximate equilibrium.
/// See docs/research-solution-methods.md, section 3.
pub struct SmMcts {
    allocations_a: Vec<Allocation>,
    allocations_b: Vec<Allocation>,
    nodes: Vec<Node>,
    /// Probability of a uniformly random action at each node (exploration).
    exploration: f64,
    rng: rand::rngs::StdRng,
}

struct Node {
    regrets_a: Vec<f64>,
    regrets_b: Vec<f64>,
    strategy_sum_a: Vec<f64>,
    strategy_sum_b: Vec<f64>,
    children: HashMap<(usize, usize), usize>,
}

impl Node {
    fn new(actions_a: usize, actions_b: usize) -> Self {
        Self {
            regrets_a: vec![0.0; actions_a],
            regrets_b: vec![0.0; actions_b],
            strategy_sum_a: vec![0.0; actions_a],
            strategy_sum_b: vec![0.0; actions_b],
            children: HashMap::new(),
        }
    }
}

/// Regret matching: play actions in proportion to positive regret,
/// uniformly if none is positive; then mix in `exploration` uniform noise.
fn regret_matching(regrets: &[f64], exploration: f64) -> Vec<f64> {
    let n = regrets.len() as f64;
    let positive: f64 = regrets.iter().map(|r| r.max(0.0)).sum();
    regrets
        .iter()
        .map(|r| {
            let p = if positive > 0.0 { r.max(0.0) / positive } else { 1.0 / n };
            (1.0 - exploration) * p + exploration / n
        })
        .collect()
}

fn sample(probabilities: &[f64], rng: &mut impl Rng) -> usize {
    let r: f64 = rng.gen();
    let mut cumulative = 0.0;
    for (i, p) in probabilities.iter().enumerate() {
        cumulative += p;
        if r < cumulative {
            return i;
        }
    }
    probabilities.len() - 1
}

/// Purification: drop actions below `threshold` probability and renormalize.
/// The average strategy keeps residual mass from exploration and early,
/// near-uniform iterations; sampling it every turn makes the team split
/// needlessly, which is costly when switches are limited and ropes heavy.
fn purify(probabilities: &[f64], threshold: f64) -> Vec<f64> {
    let kept: Vec<f64> = probabilities.iter().map(|&p| if p >= threshold { p } else { 0.0 }).collect();
    let total: f64 = kept.iter().sum();
    if total <= 0.0 {
        return probabilities.to_vec();
    }
    kept.iter().map(|p| p / total).collect()
}

fn normalize(sums: &[f64]) -> Vec<f64> {
    let total: f64 = sums.iter().sum();
    if total <= 0.0 {
        return vec![1.0 / sums.len() as f64; sums.len()];
    }
    sums.iter().map(|s| s / total).collect()
}

impl SmMcts {
    pub fn new(state: &GameState, seed: u64) -> Self {
        let ropes = state.ropes.len();
        Self {
            allocations_a: generate_allocations(state.get_team_player_count(Team::A), ropes),
            allocations_b: generate_allocations(state.get_team_player_count(Team::B), ropes),
            nodes: Vec::new(),
            exploration: 0.1,
            rng: rand::rngs::StdRng::seed_from_u64(seed),
        }
    }

    /// Run `iterations` samples from `state` (a fresh tree) and return the
    /// root's average strategies for Team A and Team B.
    pub fn search(&mut self, state: &GameState, iterations: usize) -> (Vec<f64>, Vec<f64>) {
        self.nodes.clear();
        self.nodes.push(Node::new(self.allocations_a.len(), self.allocations_b.len()));
        for _ in 0..iterations {
            self.iterate(0, state.clone());
        }
        let root = &self.nodes[0];
        (normalize(&root.strategy_sum_a), normalize(&root.strategy_sum_b))
    }

    pub fn allocations(&self, team: Team) -> &[Allocation] {
        match team {
            Team::A => &self.allocations_a,
            Team::B => &self.allocations_b,
        }
    }

    /// One sampled path from `node`. Returns the payoff for Team A
    /// (final score differential A - B).
    fn iterate(&mut self, node: usize, mut state: GameState) -> f64 {
        if state.finished() {
            return score_differential(&state);
        }

        let sigma_a = regret_matching(&self.nodes[node].regrets_a, self.exploration);
        let sigma_b = regret_matching(&self.nodes[node].regrets_b, self.exploration);
        let i = sample(&sigma_a, &mut self.rng);
        let j = sample(&sigma_b, &mut self.rng);

        self.apply(&mut state, i, j);

        let u = match self.nodes[node].children.get(&(i, j)).copied() {
            Some(child) => self.iterate(child, state),
            None => {
                // Expand one node per iteration, then estimate by random playout.
                let child = self.nodes.len();
                self.nodes.push(Node::new(self.allocations_a.len(), self.allocations_b.len()));
                self.nodes[node].children.insert((i, j), child);
                self.rollout(state)
            }
        };

        // Importance-weighted estimate of each action's payoff: only the
        // sampled action has an observation, scaled by 1 / its probability.
        let n = &mut self.nodes[node];
        for a in 0..n.regrets_a.len() {
            let estimate = if a == i { u / sigma_a[i] } else { 0.0 };
            n.regrets_a[a] += estimate - u;
            n.strategy_sum_a[a] += sigma_a[a];
        }
        // Team B minimizes, so its payoff is -u.
        for b in 0..n.regrets_b.len() {
            let estimate = if b == j { -u / sigma_b[j] } else { 0.0 };
            n.regrets_b[b] += estimate + u;
            n.strategy_sum_b[b] += sigma_b[b];
        }
        u
    }

    fn rollout(&mut self, mut state: GameState) -> f64 {
        while !state.finished() {
            let i = self.rng.gen_range(0..self.allocations_a.len());
            let j = self.rng.gen_range(0..self.allocations_b.len());
            self.apply(&mut state, i, j);
        }
        score_differential(&state)
    }

    fn apply(&self, state: &mut GameState, i: usize, j: usize) {
        let action_a = allocation_to_action(state, Team::A, &self.allocations_a[i].counts);
        let action_b = allocation_to_action(state, Team::B, &self.allocations_b[j].counts);
        step(state, &action_a, &action_b);
    }
}

fn score_differential(state: &GameState) -> f64 {
    let score = calculate_score(state);
    (score.team_a - score.team_b) as f64
}

const PURIFICATION_THRESHOLD: f64 = 0.15;

/// Plays SM-MCTS for either team: searches from the current state each
/// turn and samples an allocation from the root's average strategy.
pub struct SmMctsStrategy {
    iterations: usize,
    seed: u64,
    search: Option<SmMcts>,
}

impl SmMctsStrategy {
    pub fn new(seed: u64, iterations: usize) -> Self {
        Self { iterations, seed, search: None }
    }
}

impl Strategy for SmMctsStrategy {
    fn decide(&mut self, state: &GameState, team: Team) -> TeamAction {
        let seed = self.seed;
        let search = self.search.get_or_insert_with(|| SmMcts::new(state, seed));
        let (sigma_a, sigma_b) = search.search(state, self.iterations);
        let sigma = match team {
            Team::A => sigma_a,
            Team::B => sigma_b,
        };
        let choice = sample(&purify(&sigma, PURIFICATION_THRESHOLD), &mut search.rng);
        allocation_to_action(state, team, &search.allocations(team)[choice].counts)
    }

    fn name(&self) -> &str {
        "SM-MCTS"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::algorithms::ShapleyDpSolver;
    use crate::game::create_uneven_arena_game;

    #[test]
    fn regret_matching_follows_positive_regret() {
        let p = regret_matching(&[3.0, 1.0, -5.0], 0.0);
        assert!((p[0] - 0.75).abs() < 1e-9);
        assert!((p[1] - 0.25).abs() < 1e-9);
        assert_eq!(p[2], 0.0);
        let uniform = regret_matching(&[-1.0, -2.0], 0.0);
        assert_eq!(uniform, vec![0.5, 0.5]);
    }

    /// On a one-turn game, the payoff matrix is exact, so we can measure how
    /// far SM-MCTS's average strategies are from the Shapley value against a
    /// best-responding opponent (exploitability).
    #[test]
    fn one_turn_strategies_are_close_to_shapley_value() {
        let mut state = create_uneven_arena_game();
        state.rules.duration = 1;
        // Make B's two players unequal so the equilibrium is nontrivial.
        state.players[3].strength = 9.0;
        state.players[4].strength = 5.0;

        let value = ShapleyDpSolver::new(&state).solve(&state);

        let mut mcts = SmMcts::new(&state, 7);
        let (x, y) = mcts.search(&state, 20_000);
        let payoff: Vec<Vec<f64>> = (0..x.len())
            .map(|i| {
                (0..y.len())
                    .map(|j| {
                        let mut s = state.clone();
                        mcts.apply(&mut s, i, j);
                        score_differential(&s)
                    })
                    .collect()
            })
            .collect();

        // A's guarantee: worst column against x. B's guarantee: best row against y.
        let a_guarantee = (0..y.len())
            .map(|j| (0..x.len()).map(|i| x[i] * payoff[i][j]).sum::<f64>())
            .fold(f64::INFINITY, f64::min);
        let b_guarantee = (0..x.len())
            .map(|i| (0..y.len()).map(|j| y[j] * payoff[i][j]).sum::<f64>())
            .fold(f64::NEG_INFINITY, f64::max);

        assert!(a_guarantee >= value - 0.1, "A {} vs value {}", a_guarantee, value);
        assert!(b_guarantee <= value + 0.1, "B {} vs value {}", b_guarantee, value);
    }
}
