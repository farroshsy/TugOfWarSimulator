use crate::game::{GameState, Team};
use crate::strategies::{Strategy, TeamAction, allocation_to_action};

/// Fatigue-aware strategy: considers remaining stamina and game phase
pub struct FatigueAwareStrategy;

impl FatigueAwareStrategy {
    pub fn new() -> Self {
        Self
    }
    
    /// Calculate game phase (0.0 = early, 1.0 = late)
    fn game_phase(&self, state: &GameState) -> f64 {
        if state.rules.duration == 0 {
            return 1.0;
        }
        state.time as f64 / state.rules.duration as f64
    }
    
    /// Calculate rope priority considering fatigue
    fn calculate_rope_priority(&self, state: &GameState, team: Team, rope: &crate::game::Rope, team_energy: f64) -> f64 {
        let phase = self.game_phase(state);
        
        // Value is always important
        let value_factor = rope.value as f64 * 2.0;
        
        // Position bonus
        let positional_bonus = match team {
            Team::A => (-rope.position) as f64,
            Team::B => rope.position as f64,
        };
        
        // Early game: conserve energy, be conservative
        // Late game: commit heavily to winnable ropes
        let energy_factor = if phase < 0.3 {
            // Early game: prefer ropes we already control
            if (team == Team::A && rope.position < 0) || (team == Team::B && rope.position > 0) {
                1.0
            } else {
                0.5
            }
        } else if phase > 0.7 {
            // Late game: go for highest value regardless of position
            1.5
        } else {
            // Middle game: balanced approach
            1.0
        };
        
        // If energy is low, reduce priority of aggressive moves
        let fatigue_penalty = if team_energy < 30.0 {
            0.5
        } else {
            1.0
        };
        
        (value_factor + positional_bonus * 0.3) * energy_factor * fatigue_penalty
    }
}

impl Strategy for FatigueAwareStrategy {
    fn decide(&mut self, state: &GameState, team: Team) -> TeamAction {
        let team_energy = state.get_team_energy(team);
        let phase = self.game_phase(state);
        
        // If energy is very low and it's not late game, conserve
        if team_energy < 20.0 && phase < 0.8 {
            // Return empty action (no movement)
            return TeamAction::empty();
        }
        
        // Calculate priority for each rope
        let mut priorities = vec![0.0; state.ropes.len()];
        
        for rope in &state.ropes {
            priorities[rope.id] = self.calculate_rope_priority(state, team, rope, team_energy);
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
        "FatigueAware"
    }
}