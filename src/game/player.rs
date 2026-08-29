use serde::{Deserialize, Serialize};

pub type PlayerId = usize;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Team {
    A,
    B,
}

impl Team {
    pub fn opponent(self) -> Team {
        match self {
            Team::A => Team::B,
            Team::B => Team::A,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Role {
    Normal,
    Commander,
    Spy,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum PlayerMode {
    Pull,
    Rest,
    Guard,
    Observe,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Player {
    pub id: PlayerId,
    pub team: Team,
    
    // Physical attributes
    pub strength: f64,
    pub stamina: f64,
    pub max_stamina: f64,
    
    // Fatigue parameters
    pub fatigue_rate: f64,
    pub recovery_rate: f64,
    
    // Role and behavior
    pub role: Role,
    pub mode: PlayerMode,
    
    // Current assignment
    pub current_rope: usize,
    
    // Information model
    pub local_information: bool,
}

impl Player {
    pub fn effective_strength(&self) -> f64 {
        if self.max_stamina <= 0.0 {
            return 0.0;
        }

        let stamina_ratio = (self.stamina / self.max_stamina).clamp(0.0, 1.0);
        
        // Pull mode contributes full effective strength
        // Other modes may contribute less (for future expansion)
        match self.mode {
            PlayerMode::Pull => self.strength * stamina_ratio,
            PlayerMode::Rest => 0.0,
            PlayerMode::Guard => self.strength * stamina_ratio * 0.5,
            PlayerMode::Observe => 0.0,
        }
    }
    
    pub fn is_fatigued(&self, threshold: f64) -> bool {
        self.stamina < self.max_stamina * threshold
    }
}