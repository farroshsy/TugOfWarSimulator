use crate::game::{GameState, Team};
use crate::strategies::{Strategy, TeamAction, allocation_to_action};

/// Greedy strategy: always allocate strongest players to highest-value rope
pub struct GreedyStrategy;

impl GreedyStrategy {
    pub fn new() -> Self {
        Self
    }
}

impl Strategy for GreedyStrategy {
    fn decide(&mut self, state: &GameState, team: Team) -> TeamAction {
        // Find the highest-value rope
        let mut best_rope = 0;
        let mut best_value = -1;
        
        for rope in &state.ropes {
            if rope.value > best_value {
                best_value = rope.value;
                best_rope = rope.id;
            }
        }
        
        // Allocate all players to the best rope
        let num_players = state.get_team_player_count(team);
        let mut allocation = vec![0; state.ropes.len()];
        allocation[best_rope] = num_players;
        
        allocation_to_action(state, team, &allocation)
    }
    
    fn name(&self) -> &str {
        "Greedy"
    }
}