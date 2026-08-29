use crate::game::{GameState, Team};

/// Switching constraints and cost calculation
pub struct SwitchingManager {
    max_switches_per_turn: usize,
    switching_stamina_cost: f64,
    enabled: bool,
}

impl SwitchingManager {
    pub fn new(max_switches: usize, cost: f64, enabled: bool) -> Self {
        Self {
            max_switches_per_turn: max_switches,
            switching_stamina_cost: cost,
            enabled,
        }
    }
    
    /// Calculate how many switches are possible in current state
    pub fn available_switches(&self, state: &GameState, team: Team) -> usize {
        if !self.enabled {
            return usize::MAX; // Unlimited if disabled
        }
        
        // Players with sufficient stamina can switch
        state.players.iter()
            .filter(|p| p.team == team && p.stamina >= self.switching_stamina_cost)
            .count()
            .min(self.max_switches_per_turn)
    }
    
    /// Check if a switch is valid
    pub fn can_switch(&self, state: &GameState, player_id: usize, _from: usize, _to: usize) -> bool {
        if !self.enabled {
            return true;
        }
        
        if let Some(player) = state.players.get(player_id) {
            player.stamina >= self.switching_stamina_cost
        } else {
            false
        }
    }
    
    /// Apply switching cost to a player
    pub fn apply_switching_cost(&self, player: &mut crate::game::Player) {
        if self.enabled {
            player.stamina = (player.stamina - self.switching_stamina_cost).max(0.0);
        }
    }
    
    /// Get total switching cost for a team
    pub fn total_switching_cost(&self, state: &GameState, team: Team) -> f64 {
        if !self.enabled {
            return 0.0;
        }
        
        state.players.iter()
            .filter(|p| p.team == team)
            .filter(|p| p.stamina >= self.switching_stamina_cost)
            .count() as f64 * self.switching_stamina_cost
    }
}