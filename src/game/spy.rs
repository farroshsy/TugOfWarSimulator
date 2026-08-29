use crate::game::{GameState, Team, PlayerId, RopeId};
use rand::Rng;

/// Spy behavior types
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SpyBehavior {
    /// Spy always acts against own team
    Deterministic,
    /// Spy acts against own team with probability p
    Stochastic(f64),
    /// Spy chooses behavior based on game state
    Adaptive,
}

/// Spy system for adversarial internal agents
pub struct SpySystem {
    enabled: bool,
    spy_probability: f64, // Probability a player is a spy
    behavior: SpyBehavior,
    known_spies: Vec<PlayerId>, // Spies that are known to the team
    hidden_spies: Vec<PlayerId>, // Spies that are hidden
}

impl SpySystem {
    pub fn new(enabled: bool, spy_prob: f64, behavior: SpyBehavior) -> Self {
        Self {
            enabled,
            spy_probability: spy_prob,
            behavior,
            known_spies: Vec::new(),
            hidden_spies: Vec::new(),
        }
    }
    
    /// Initialize spies randomly based on probability
    pub fn initialize_spies(&mut self, state: &mut GameState, rng: &mut impl rand::Rng) {
        if !self.enabled {
            return;
        }
        
        self.hidden_spies.clear();
        self.known_spies.clear();
        
        for player in &state.players {
            if rng.gen_bool(self.spy_probability) {
                self.hidden_spies.push(player.id);
                // Mark as spy in player (but this may be hidden from strategy)
                if let Some(p) = state.players.get_mut(player.id) {
                    p.role = crate::game::Role::Spy;
                }
            }
        }
    }
    
    /// Check if a player is a spy
    pub fn is_spy(&self, player_id: PlayerId) -> bool {
        self.hidden_spies.contains(&player_id) || self.known_spies.contains(&player_id)
    }
    
    /// Check if spy identity is known to the team
    pub fn is_known_spy(&self, player_id: PlayerId) -> bool {
        self.known_spies.contains(&player_id)
    }
    
    /// Apply spy behavior to the game state
    pub fn apply_spy_behavior(&mut self, state: &mut GameState, rng: &mut impl rand::Rng) {
        if !self.enabled {
            return;
        }
        
        for &spy_id in &self.hidden_spies {
            if let Some(player) = state.players.get(spy_id) {
                if player.role != crate::game::Role::Spy {
                    continue;
                }
                
                // Determine if spy should act maliciously
                let should_betray = match self.behavior {
                    SpyBehavior::Deterministic => true,
                    SpyBehavior::Stochastic(prob) => rng.gen_bool(prob),
                    SpyBehavior::Adaptive => self.should_adaptive_betray(state, player.team, rng),
                };
                
                if should_betray {
                    self.apply_betrayal(state, spy_id, player.team);
                }
            }
        }
    }
    
    /// Adaptive spy behavior based on game state
    fn should_adaptive_betray(&self, state: &GameState, team: Team, rng: &mut impl rand::Rng) -> bool {
        // More likely to betray when it can swing the game
        let score = crate::game::simulation::calculate_score(state);
        let current_advantage = match team {
            Team::A => score.team_a - score.team_b,
            Team::B => score.team_b - score.team_a,
        };
        
        // If team is winning, spy is more likely to betray
        // If team is losing, spy might try to help (or continue betraying)
        if current_advantage > 0 {
            rng.gen_bool(0.7) // High betrayal probability when winning
        } else {
            rng.gen_bool(0.3) // Lower betrayal probability when losing
        }
    }
    
    /// Apply betrayal action
    fn apply_betrayal(&self, state: &mut GameState, spy_id: PlayerId, team: Team) {
        // Find rope that helps opponent most
        let mut worst_rope_for_team = 0;
        let mut worst_value = -1;
        
        for rope in &state.ropes {
            let is_bad_for_team = match team {
                Team::A => rope.position > 0,
                Team::B => rope.position < 0,
            };
            
            if is_bad_for_team && rope.value > worst_value {
                worst_value = rope.value;
                worst_rope_for_team = rope.id;
            }
        }
        
        // Move spy to rope that hurts their team
        if worst_rope_for_team < state.ropes.len() {
            state.assignments[spy_id] = worst_rope_for_team;
            if let Some(p) = state.players.get_mut(spy_id) {
                p.current_rope = worst_rope_for_team;
            }
        }
        
        state.rebuild_rope_counts();
    }
    
    /// Reveal a spy (for debugging or special events)
    pub fn reveal_spy(&mut self, player_id: PlayerId) {
        if self.hidden_spies.contains(&player_id) {
            self.hidden_spies.retain(|&id| id != player_id);
            self.known_spies.push(player_id);
        }
    }
    
    /// Get spy statistics
    pub fn spy_stats(&self) -> (usize, usize) {
        (self.hidden_spies.len(), self.known_spies.len())
    }
}