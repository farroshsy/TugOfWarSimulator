use crate::game::{GameState, Team};
use crate::strategies::{Strategy, TeamAction, allocation_to_action, Allocation};
use crate::algorithms::generate_allocations;
use crate::game::simulation::{step, calculate_score};
use std::collections::HashMap;
use std::sync::Arc;

/// Fatigue-aware DP state with compressed energy representation
#[derive(Hash, Eq, PartialEq, Clone, Debug)]
struct FatigueStateKey {
    time: usize,
    positions: Vec<i32>,
    energy_a: u8,  // Compressed to 0-20 buckets
    energy_b: u8,  // Compressed to 0-20 buckets
}

/// Fatigue-aware DP solver with compressed energy state
pub struct FatigueDpSolver {
    cache: HashMap<FatigueStateKey, f64>,
    // Arc: cheap to hold across the recursive &mut self calls.
    allocations_a: Arc<[Allocation]>,
    allocations_b: Arc<[Allocation]>,
    policy_cache: HashMap<FatigueStateKey, Allocation>,
    energy_buckets: u8,  // Number of energy discretization buckets
}

impl FatigueDpSolver {
    pub fn new(people_per_team: usize, ropes: usize, energy_buckets: u8) -> Self {
        Self::for_teams(people_per_team, people_per_team, ropes, energy_buckets)
    }
    
    /// Solver for teams of different sizes.
    pub fn for_teams(people_a: usize, people_b: usize, ropes: usize, energy_buckets: u8) -> Self {
        Self {
            cache: HashMap::new(),
            allocations_a: generate_allocations(people_a, ropes).into(),
            allocations_b: generate_allocations(people_b, ropes).into(),
            policy_cache: HashMap::new(),
            energy_buckets,
        }
    }
    
    /// Compress continuous energy into discrete buckets
    fn compress_energy(energy: f64, max_energy: f64, buckets: u8) -> u8 {
        if max_energy <= 0.0 {
            return 0;
        }
        
        let ratio = (energy / max_energy).clamp(0.0, 1.0);
        ((ratio * (buckets as f64 - 1.0)).round() as u8).clamp(0, buckets - 1)
    }
    
    /// Solve the fatigue-aware DP
    pub fn solve(&mut self, state: &GameState) -> f64 {
        let max_energy_a = state.get_team_max_energy(Team::A);
        let max_energy_b = state.get_team_max_energy(Team::B);
        
        let energy_a = Self::compress_energy(
            state.get_team_energy(Team::A),
            max_energy_a,
            self.energy_buckets
        );
        
        let energy_b = Self::compress_energy(
            state.get_team_energy(Team::B),
            max_energy_b,
            self.energy_buckets
        );
        
        let key = FatigueStateKey {
            time: state.time,
            positions: state.ropes.iter().map(|r| r.position).collect(),
            energy_a,
            energy_b,
        };
        
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
        
        let allocations_a = Arc::clone(&self.allocations_a);
        let allocations_b = Arc::clone(&self.allocations_b);
        
        for allocation_a in allocations_a.iter() {
            let mut worst_value = f64::INFINITY;
            
            for allocation_b in allocations_b.iter() {
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
        
        if let Some(alloc) = best_allocation {
            self.policy_cache.insert(key.clone(), alloc);
        }
        
        self.cache.insert(key, best_value);
        best_value
    }
    
    /// Get optimal action for fatigue-aware DP
    pub fn get_optimal_action(&mut self, state: &GameState) -> Option<Allocation> {
        self.solve(state);
        
        let max_energy_a = state.get_team_max_energy(Team::A);
        let max_energy_b = state.get_team_max_energy(Team::B);
        
        let energy_a = Self::compress_energy(
            state.get_team_energy(Team::A),
            max_energy_a,
            self.energy_buckets
        );
        
        let energy_b = Self::compress_energy(
            state.get_team_energy(Team::B),
            max_energy_b,
            self.energy_buckets
        );
        
        let key = FatigueStateKey {
            time: state.time,
            positions: state.ropes.iter().map(|r| r.position).collect(),
            energy_a,
            energy_b,
        };
        
        self.policy_cache.get(&key).cloned()
    }
    
    pub fn clear_cache(&mut self) {
        self.cache.clear();
        self.policy_cache.clear();
    }
}

/// Fatigue-aware DP strategy
pub struct FatigueDpStrategy {
    solver: FatigueDpSolver,
}

impl FatigueDpStrategy {
    pub fn new(people_per_team: usize, ropes: usize, energy_buckets: u8) -> Self {
        Self::for_teams(people_per_team, people_per_team, ropes, energy_buckets)
    }
    
    pub fn for_teams(people_a: usize, people_b: usize, ropes: usize, energy_buckets: u8) -> Self {
        Self {
            solver: FatigueDpSolver::for_teams(people_a, people_b, ropes, energy_buckets),
        }
    }
}

impl Strategy for FatigueDpStrategy {
    fn decide(&mut self, state: &GameState, team: Team) -> TeamAction {
        if team != Team::A {
            return crate::strategies::greedy::GreedyStrategy::new()
                .decide(state, team);
        }
        
        if let Some(allocation) = self.solver.get_optimal_action(state) {
            allocation_to_action(state, team, &allocation.counts)
        } else {
            crate::strategies::greedy::GreedyStrategy::new()
                .decide(state, team)
        }
    }
    
    fn name(&self) -> &str {
        "FatigueDP"
    }
}