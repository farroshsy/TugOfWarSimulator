use crate::game::{GameState, Player, Rope, GameRules, Team, Role, PlayerMode};

/// Create a default game configuration
pub fn create_default_game() -> GameState {
    let rules = GameRules::default();
    
    let ropes = vec![
        Rope::new(0, 0, -10, 10, 1.0, 1),
        Rope::new(1, 0, -10, 10, 2.0, 2),
        Rope::new(2, 0, -10, 10, 3.0, 3),
    ];
    
    let mut players = Vec::new();
    
    // Team A: 12 players
    for i in 0..12 {
        players.push(Player {
            id: i,
            team: Team::A,
            strength: if i < 3 { 10.0 } else { 7.0 },
            stamina: 100.0,
            max_stamina: 100.0,
            fatigue_rate: if i < 3 { 1.8 } else { 1.2 },
            recovery_rate: 0.8,
            role: if i == 0 { Role::Commander } else { Role::Normal },
            mode: PlayerMode::Pull,
            current_rope: 0,
            local_information: false,
        });
    }
    
    // Team B: 12 players
    for i in 0..12 {
        let id = 12 + i;
        players.push(Player {
            id,
            team: Team::B,
            strength: if i < 4 { 8.5 } else { 6.5 },
            stamina: 100.0,
            max_stamina: 100.0,
            fatigue_rate: if i < 4 { 1.3 } else { 1.0 },
            recovery_rate: 0.8,
            role: if i == 0 { Role::Commander } else { Role::Normal },
            mode: PlayerMode::Pull,
            current_rope: 0,
            local_information: false,
        });
    }
    
    GameState::new(players, ropes, rules, 42)
}

/// Create a small game for DP testing (3v3, 2 ropes, 10 turns)
pub fn create_small_game() -> GameState {
    let rules = GameRules::small_game();
    
    let ropes = vec![
        Rope::new(0, 0, -5, 5, 1.0, 1),
        Rope::new(1, 0, -5, 5, 2.0, 2),
    ];
    
    let mut players = Vec::new();
    
    // Team A: 3 players
    for i in 0..3 {
        players.push(Player {
            id: i,
            team: Team::A,
            strength: 7.0,
            stamina: 100.0,
            max_stamina: 100.0,
            fatigue_rate: 1.0,
            recovery_rate: 0.5,
            role: Role::Normal,
            mode: PlayerMode::Pull,
            current_rope: 0,
            local_information: false,
        });
    }
    
    // Team B: 3 players
    for i in 0..3 {
        let id = 3 + i;
        players.push(Player {
            id,
            team: Team::B,
            strength: 7.0,
            stamina: 100.0,
            max_stamina: 100.0,
            fatigue_rate: 1.0,
            recovery_rate: 0.5,
            role: Role::Normal,
            mode: PlayerMode::Pull,
            current_rope: 0,
            local_information: false,
        });
    }
    
    GameState::new(players, ropes, rules, 123)
}