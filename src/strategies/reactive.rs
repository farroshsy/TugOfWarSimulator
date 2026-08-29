use crate::game::{GameState, Team};
use crate::strategies::{Strategy, TeamAction, allocation_to_action};

/// Reactive strategy: considers current rope position and opponent allocation
pub struct ReactiveStrategy;

impl ReactiveStrategy {
    pub fn new() -> Self {
        Self
    }
    
    /// Calculate priority score for each rope
    fn calculate_rope_priority(&self, _state: &GameState, team: Team, rope: &crate::game::Rope) -> f64 {
        let positional_bonus = match team {
            Team::A => (-rope.position) as f64,  // Prefer ropes in A's territory
            Team::B => rope.position as f64,     // Prefer ropes in B's territory
        };
        
        // Value is the primary factor
        let value_factor = rope.value as f64 * 2.0;
        
        // Position gives bonus for ropes closer to opponent's territory
        let position_factor = positional_bonus * 0.3;
        
        // Consider opponent allocation - heavily contested ropes get higher priority
        let opponent_count = match team {
            Team::A => rope.team_b_count,
            Team::B => rope.team_a_count,
        };
        let contest_factor = opponent_count as f64 * 0.2;
        
        value_factor + position_factor + contest_factor
    }
}

impl Strategy for ReactiveStrategy {
    fn decide(&mut self, state: &GameState, team: Team) -> TeamAction {
        // Calculate priority for each rope
        let mut priorities = vec![0.0; state.ropes.len()];
        
        for rope in &state.ropes {
            priorities[rope.id] = self.calculate_rope_priority(state, team, rope);
        }
        
        // Find the highest priority rope
        let target = priorities
            .iter()
            .enumerate()
            .max_by(|a, b| a.1.partial_cmp(b.1).unwrap())
            .map(|(i, _)| i)
            .unwrap_or(0);
        
        // Allocate all players to the target rope
        let num_players = state.get_team_player_count(team);
        let mut allocation = vec![0; state.ropes.len()];
        allocation[target] = num_players;
        
        allocation_to_action(state, team, &allocation)
    }
    
    fn name(&self) -> &str {
        "Reactive"
    }
}