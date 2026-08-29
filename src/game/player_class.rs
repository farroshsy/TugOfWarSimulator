use crate::game::Player;

/// Player class for heterogeneous player modeling
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PlayerClass {
    Strong,   // High strength, moderate stamina
    Normal,   // Balanced attributes
    Weak,     // Low strength, but high stamina
}

impl PlayerClass {
    /// Create a player with class-specific attributes
    pub fn create_player(id: usize, team: crate::game::Team, class: PlayerClass) -> Player {
        match class {
            PlayerClass::Strong => Player {
                id,
                team,
                strength: 10.0,
                stamina: 100.0,
                max_stamina: 100.0,
                fatigue_rate: 1.8,
                recovery_rate: 0.6,
                role: crate::game::Role::Normal,
                mode: crate::game::PlayerMode::Pull,
                current_rope: 0,
                local_information: false,
            },
            PlayerClass::Normal => Player {
                id,
                team,
                strength: 7.0,
                stamina: 100.0,
                max_stamina: 100.0,
                fatigue_rate: 1.2,
                recovery_rate: 0.8,
                role: crate::game::Role::Normal,
                mode: crate::game::PlayerMode::Pull,
                current_rope: 0,
                local_information: false,
            },
            PlayerClass::Weak => Player {
                id,
                team,
                strength: 5.0,
                stamina: 120.0,  // Higher stamina for endurance
                max_stamina: 120.0,
                fatigue_rate: 0.8,  // Lower fatigue rate
                recovery_rate: 1.2,  // Higher recovery
                role: crate::game::Role::Normal,
                mode: crate::game::PlayerMode::Pull,
                current_rope: 0,
                local_information: false,
            },
        }
    }
    
    /// Get class name
    pub fn name(&self) -> &str {
        match self {
            PlayerClass::Strong => "Strong",
            PlayerClass::Normal => "Normal",
            PlayerClass::Weak => "Weak",
        }
    }
}