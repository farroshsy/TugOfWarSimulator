use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct GameRules {
    // Time configuration
    pub duration: usize,           // Total game duration in time steps
    pub dt: f64,                   // Time step size in seconds
    
    // Position configuration
    pub position_range: i32,       // Maximum distance from center (+/-)
    
    // Switching constraints
    pub max_switches_per_turn: usize,
    pub switching_stamina_cost: f64,
    
    // Physics model
    pub use_weight_threshold: bool,
    pub movement_scale: f64,
    
    // Commander mechanics
    pub command_cooldown: usize,    // Minimum turns between commands
    pub command_delay: usize,      // Turns for command to take effect
    
    // Special features
    pub fatigue_enabled: bool,
    pub switching_enabled: bool,
    pub commander_enabled: bool,
    pub spies_enabled: bool,
    
    // Information model
    pub perfect_information: bool,
}

impl Default for GameRules {
    fn default() -> Self {
        Self {
            duration: 60,
            dt: 1.0,
            position_range: 10,
            max_switches_per_turn: 4,
            switching_stamina_cost: 2.0,
            use_weight_threshold: true,
            movement_scale: 1.0,
            command_cooldown: 3,
            command_delay: 1,
            fatigue_enabled: true,
            switching_enabled: true,
            commander_enabled: false,
            spies_enabled: false,
            perfect_information: true,
        }
    }
}

impl GameRules {
    pub fn small_game() -> Self {
        Self {
            duration: 10,
            dt: 1.0,
            position_range: 5,
            max_switches_per_turn: 2,
            switching_stamina_cost: 1.0,
            use_weight_threshold: true,
            movement_scale: 1.0,
            command_cooldown: 2,
            command_delay: 1,
            fatigue_enabled: false,
            switching_enabled: true,
            commander_enabled: false,
            spies_enabled: false,
            perfect_information: true,
        }
    }
}