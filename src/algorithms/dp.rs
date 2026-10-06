use crate::game::{GameState, Team};
use crate::strategies::{Strategy, TeamAction, allocation_to_action, Allocation};
use crate::algorithms::generate_allocations;
use crate::game::simulation::{step, calculate_score};
use std::collections::HashMap;

/// State key for DP memoization.
///
/// Must contain everything that affects future transitions. Rope positions
/// alone are not enough: switching costs stamina (even with fatigue off),
/// stamina scales pulling force, and the per-turn switch limit makes the
/// reachable allocations depend on where players currently stand.
/// Player strengths are fixed for a game, so a solver is per-game.
/// Assumes commander and spies are disabled.
#[derive(Hash, Eq, PartialEq, Clone, Debug)]
pub(crate) struct StateKey {
    time: usize,
    positions: Vec<i32>,
    assignments: Vec<usize>,
    stamina_bits: Vec<u64>,
}

impl StateKey {
    pub(crate) fn from_state(state: &GameState) -> Self {
        Self {
            time: state.time,
            positions: state.ropes.iter().map(|r| r.position).collect(),
            assignments: state.assignments.clone(),
            stamina_bits: state.players.iter().map(|p| p.stamina.to_bits()).collect(),
        }
    }
}

/// Exact minimax DP solver for small state spaces
pub struct DpSolver {
    cache: HashMap<StateKey, f64>,
    allocations_a: Vec<Allocation>,
    allocations_b: Vec<Allocation>,
    policy_cache: HashMap<StateKey, Allocation>, // Store optimal actions
}

impl DpSolver {
    pub fn new(people_per_team: usize, ropes: usize) -> Self {
        Self::for_teams(people_per_team, people_per_team, ropes)
    }
    
    /// Solver for teams of different sizes.
    pub fn for_teams(people_a: usize, people_b: usize, ropes: usize) -> Self {
        Self {
            cache: HashMap::new(),
            allocations_a: generate_allocations(people_a, ropes),
            allocations_b: generate_allocations(people_b, ropes),
            policy_cache: HashMap::new(),
        }
    }
    
    /// Solve the DP and return the optimal value for Team A from the current state
    pub fn solve(&mut self, state: &GameState) -> f64 {
        let key = StateKey::from_state(state);
        
        if let Some(&value) = self.cache.get(&key) {
            return value;
        }
        
        if state.finished() {
            let score = calculate_score(state);
            let result = (score.team_a - score.team_b) as f64;
            self.cache.insert(key, result);
            return result;
        }
        
        let mut best_value = f64::NEG_INFINITY;
        let mut best_allocation = None;
        
        // Clone allocations to avoid borrowing issues
        let allocations_a = self.allocations_a.clone();
        let allocations_b = self.allocations_b.clone();
        
        // Team A chooses allocation to maximize
        for allocation_a in &allocations_a {
            let mut worst_value = f64::INFINITY;
            
            // Team B chooses allocation to minimize Team A's result
            for allocation_b in &allocations_b {
                let mut next_state = state.clone();
                
                let action_a = allocation_to_action(&next_state, Team::A, &allocation_a.counts);
                let action_b = allocation_to_action(&next_state, Team::B, &allocation_b.counts);
                
                step(&mut next_state, &action_a, &action_b);
                
                let value = self.solve(&next_state);
                worst_value = worst_value.min(value);
            }
            
            if worst_value > best_value {
                best_value = worst_value;
                best_allocation = Some(allocation_a.clone());
            }
        }
        
        // Store the optimal action for this state
        if let Some(alloc) = best_allocation {
            self.policy_cache.insert(key.clone(), alloc);
        }
        
        self.cache.insert(key, best_value);
        best_value
    }
    
    /// Get the optimal action for Team A from the current state
    pub fn get_optimal_action(&mut self, state: &GameState) -> Option<Allocation> {
        // First ensure this state is solved
        self.solve(state);
        
        let key = StateKey::from_state(state);
        
        self.policy_cache.get(&key).cloned()
    }
    
    /// Clear the cache (useful for reusing the solver with different parameters)
    pub fn clear_cache(&mut self) {
        self.cache.clear();
        self.policy_cache.clear();
    }
    
    /// Get cache statistics
    pub fn cache_stats(&self) -> (usize, usize) {
        (self.cache.len(), self.policy_cache.len())
    }
}

/// Feasibility estimator for DP state space
pub struct DpFeasibilityEstimator {
    people_per_team: usize,
    ropes: usize,
    time_steps: usize,
    position_range: i32,
}

impl DpFeasibilityEstimator {
    pub fn new(people_per_team: usize, ropes: usize, time_steps: usize, position_range: i32) -> Self {
        Self {
            people_per_team,
            ropes,
            time_steps,
            position_range,
        }
    }
    
    /// Estimate the number of states
    pub fn estimate_states(&self) -> u64 {
        let positions_per_rope = (2 * self.position_range + 1) as u64;
        positions_per_rope.pow(self.ropes as u32) * self.time_steps as u64
    }
    
    /// Estimate the number of allocations per team
    pub fn estimate_allocations(&self) -> u64 {
        binomial_coefficient(self.people_per_team + self.ropes - 1, self.ropes - 1) as u64
    }
    
    /// Estimate the total number of transitions
    pub fn estimate_transitions(&self) -> u64 {
        let states = self.estimate_states();
        let allocations = self.estimate_allocations();
        states * allocations * allocations
    }
    
    /// Check if DP is feasible with a given threshold
    pub fn is_feasible(&self, max_transitions: u64) -> bool {
        self.estimate_transitions() <= max_transitions
    }
    
    /// Get a feasibility report
    pub fn report(&self) -> String {
        let states = self.estimate_states();
        let allocations = self.estimate_allocations();
        let transitions = self.estimate_transitions();
        
        format!(
            "DP Feasibility Report:\n\
             States: {}\n\
             Allocations per team: {}\n\
             Joint actions per state: {}\n\
             Estimated transitions: {}\n\
             Status: {}",
            format_number(states),
            format_number(allocations),
            format_number(allocations * allocations),
            format_number(transitions),
            if transitions > 10_000_000 {
                "WARNING - Large state space"
            } else if transitions > 1_000_000 {
                "CAUTION - Moderate state space"
            } else {
                "OK - Manageable state space"
            }
        )
    }
}

fn binomial_coefficient(n: usize, k: usize) -> usize {
    if k > n {
        return 0;
    }
    if k == 0 || k == n {
        return 1;
    }
    
    let k = k.min(n - k);
    let mut result = 1;
    for i in 0..k {
        result = result * (n - i) / (i + 1);
    }
    
    result
}

fn format_number(n: u64) -> String {
    if n >= 1_000_000_000 {
        format!("{:.1} billion", n as f64 / 1_000_000_000.0)
    } else if n >= 1_000_000 {
        format!("{:.1} million", n as f64 / 1_000_000.0)
    } else if n >= 1_000 {
        format!("{:.1} thousand", n as f64 / 1_000.0)
    } else {
        n.to_string()
    }
}

/// DP Strategy that uses the exact solver
pub struct DpStrategy {
    solver: DpSolver,
    people_per_team: usize,
    ropes: usize,
}

impl DpStrategy {
    pub fn new(people_per_team: usize, ropes: usize) -> Self {
        Self::for_teams(people_per_team, people_per_team, ropes)
    }
    
    pub fn for_teams(people_a: usize, people_b: usize, ropes: usize) -> Self {
        Self {
            solver: DpSolver::for_teams(people_a, people_b, ropes),
            people_per_team: people_a,
            ropes,
        }
    }
    
    pub fn get_solver(&mut self) -> &mut DpSolver {
        &mut self.solver
    }
}

impl Strategy for DpStrategy {
    fn decide(&mut self, state: &GameState, team: Team) -> TeamAction {
        // Only works for Team A (Team A maximizes)
        if team != Team::A {
            // Fallback to greedy for Team B
            return crate::strategies::greedy::GreedyStrategy::new()
                .decide(state, team);
        }
        
        // Get optimal allocation from DP
        if let Some(allocation) = self.solver.get_optimal_action(state) {
            allocation_to_action(state, team, &allocation.counts)
        } else {
            // Fallback to greedy if DP fails
            crate::strategies::greedy::GreedyStrategy::new()
                .decide(state, team)
        }
    }
    
    fn name(&self) -> &str {
        "DP"
    }
}