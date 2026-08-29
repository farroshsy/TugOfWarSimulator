use crate::game::{GameState, Team};
use crate::strategies::{Strategy, TeamAction, allocation_to_action, Allocation};
use crate::algorithms::generate_allocations;
use crate::game::simulation::{step, calculate_score};
use std::collections::BinaryHeap;
use std::cmp::Ordering;

/// Approximate DP using limited horizon planning
pub struct LimitedHorizonDp {
    horizon: usize,
    beam_width: usize,
    people_per_team: usize,
    ropes: usize,
}

impl LimitedHorizonDp {
    pub fn new(people_per_team: usize, ropes: usize, horizon: usize, beam_width: usize) -> Self {
        Self {
            horizon,
            beam_width,
            people_per_team,
            ropes,
        }
    }
    
    /// Solve with limited horizon using beam search
    pub fn solve_limited_horizon(&self, state: &GameState, team: Team) -> f64 {
        let allocations = generate_allocations(self.people_per_team, self.ropes);
        
        if state.finished() {
            let score = calculate_score(state);
            return (score.team_a - score.team_b) as f64;
        }
        
        // Use beam search to evaluate actions over limited horizon
        let mut beam = vec![(state.clone(), 0.0)];
        
        for _ in 0..self.horizon.min(state.rules.duration - state.time) {
            let mut next_beam = Vec::new();
            
            for (current_state, current_value) in beam {
                for allocation in &allocations {
                    let mut next_state = current_state.clone();
                    let action = allocation_to_action(&next_state, team, &allocation.counts);
                    
                    // Simulate opponent with simple response
                    let opponent_action = self.get_opponent_response(&next_state, team.opponent());
                    step(&mut next_state, &action, &opponent_action);
                    
                    let score = calculate_score(&next_state);
                    let value = (score.team_a - score.team_b) as f64 + current_value;
                    
                    next_beam.push((next_state, value));
                }
            }
            
            // Keep only top beam_width states
            next_beam.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(Ordering::Equal));
            beam = next_beam.into_iter().take(self.beam_width).collect();
            
            if beam.is_empty() {
                break;
            }
        }
        
        // Return best value from beam
        beam.iter()
            .map(|(_, value)| *value)
            .fold(f64::NEG_INFINITY, f64::max)
    }
    
    /// Get best action using limited horizon evaluation
    pub fn get_best_action(&self, state: &GameState, team: Team) -> Option<Allocation> {
        let allocations = generate_allocations(self.people_per_team, self.ropes);
        
        let mut best_allocation = None;
        let mut best_value = f64::NEG_INFINITY;
        
        for allocation in &allocations {
            let mut test_state = state.clone();
            let action = allocation_to_action(&test_state, team, &allocation.counts);
            
            let opponent_action = self.get_opponent_response(&test_state, team.opponent());
            step(&mut test_state, &action, &opponent_action);
            
            let value = self.solve_limited_horizon(&test_state, team);
            
            if value > best_value {
                best_value = value;
                best_allocation = Some(allocation.clone());
            }
        }
        
        best_allocation
    }
    
    /// Simple opponent response for evaluation
    fn get_opponent_response(&self, state: &GameState, team: Team) -> TeamAction {
        // Use greedy as opponent response
        crate::strategies::greedy::GreedyStrategy::new()
            .decide(state, team)
    }
}

/// Beam search state for priority queue
#[derive(Clone, Debug)]
struct BeamState {
    state: GameState,
    value: f64,
    depth: usize,
}

impl PartialEq for BeamState {
    fn eq(&self, other: &Self) -> bool {
        self.value == other.value
    }
}

impl Eq for BeamState {}

impl PartialOrd for BeamState {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for BeamState {
    fn cmp(&self, other: &Self) -> Ordering {
        // Max-heap: reverse ordering
        other.value.partial_cmp(&self.value).unwrap_or(Ordering::Equal)
    }
}

/// Approximate DP strategy using limited horizon
pub struct ApproximateDpStrategy {
    solver: LimitedHorizonDp,
}

impl ApproximateDpStrategy {
    pub fn new(people_per_team: usize, ropes: usize, horizon: usize, beam_width: usize) -> Self {
        Self {
            solver: LimitedHorizonDp::new(people_per_team, ropes, horizon, beam_width),
        }
    }
}

impl Strategy for ApproximateDpStrategy {
    fn decide(&mut self, state: &GameState, team: Team) -> TeamAction {
        if let Some(allocation) = self.solver.get_best_action(state, team) {
            allocation_to_action(state, team, &allocation.counts)
        } else {
            crate::strategies::greedy::GreedyStrategy::new()
                .decide(state, team)
        }
    }
    
    fn name(&self) -> &str {
        "ApproxDP"
    }
}