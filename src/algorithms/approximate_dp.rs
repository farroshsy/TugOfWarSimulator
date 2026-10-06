use crate::game::{GameState, Team};
use crate::strategies::{Strategy, TeamAction, allocation_to_action, Allocation};
use crate::algorithms::generate_allocations;
use crate::game::simulation::{step, calculate_score};
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
    
    /// Solve with limited horizon using beam search.
    /// Returns the value from `team`'s perspective (own score minus opponent's).
    pub fn solve_limited_horizon(&self, state: &GameState, team: Team) -> f64 {
        let allocations = generate_allocations(state.get_team_player_count(team), self.ropes);
        
        if state.finished() {
            return own_score_differential(state, team);
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
                    step_as(&mut next_state, team, &action, &opponent_action);
                    
                    let value = own_score_differential(&next_state, team) + current_value;
                    
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
        let allocations = generate_allocations(state.get_team_player_count(team), self.ropes);
        
        let mut best_allocation = None;
        let mut best_value = f64::NEG_INFINITY;
        
        for allocation in &allocations {
            let mut test_state = state.clone();
            let action = allocation_to_action(&test_state, team, &allocation.counts);
            
            let opponent_action = self.get_opponent_response(&test_state, team.opponent());
            step_as(&mut test_state, team, &action, &opponent_action);
            
            // Count this turn's score too, not only the turns after it.
            let value = own_score_differential(&test_state, team)
                + self.solve_limited_horizon(&test_state, team);
            
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

/// `step` takes Team A's action first; order the actions by team.
fn step_as(state: &mut GameState, team: Team, own: &TeamAction, opponent: &TeamAction) {
    match team {
        Team::A => step(state, own, opponent),
        Team::B => step(state, opponent, own),
    }
}

/// Score differential from `team`'s point of view.
fn own_score_differential(state: &GameState, team: Team) -> f64 {
    let score = calculate_score(state);
    let diff = (score.team_a - score.team_b) as f64;
    match team {
        Team::A => diff,
        Team::B => -diff,
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::game::{create_arena_game, create_uneven_arena_game};

    // Against a Greedy opponent (all players on the value-2 rope), matching
    // it there ([0, 3]) gives a 0 differential; any split concedes that rope
    // for at most the value-1 rope. Both teams should find the match.
    #[test]
    fn team_a_matches_greedy_opponent_on_top_rope() {
        let state = create_arena_game();
        let dp = LimitedHorizonDp::new(3, 2, 3, 4);
        assert_eq!(dp.get_best_action(&state, Team::A).unwrap().counts, vec![0, 3]);
    }

    // Uneven arena: B (2 players) cannot win the value-2 rope against
    // Greedy A (3 players there), so B should put someone on the value-1
    // rope instead of conceding both.
    #[test]
    fn team_b_optimizes_its_own_score_not_team_a() {
        let state = create_uneven_arena_game();
        let dp = LimitedHorizonDp::new(3, 2, 3, 4);
        let counts = dp.get_best_action(&state, Team::B).unwrap().counts;
        assert!(counts[0] >= 1, "B conceded both ropes: {:?}", counts);
    }
}
