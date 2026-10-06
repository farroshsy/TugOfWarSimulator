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

/// Arena game: 3v3, 2 light ropes (values 1 and 2, weight 0.5), positions
/// -2..=2, 6 turns, free switching, no fatigue.
///
/// Designed so strategy matters: light ropes move on any force imbalance,
/// free switching lets teams fully reallocate every turn (a repeated
/// Colonel Blotto game), and the state space stays small enough for exact
/// Shapley DP (~1k states, well under a second).
pub fn create_arena_game() -> GameState {
    let mut state = create_small_game();
    state.ropes = vec![
        Rope::new(0, 0, -2, 2, 0.5, 1),
        Rope::new(1, 0, -2, 2, 0.5, 2),
    ];
    state.rules.duration = 6;
    state.rules.position_range = 2;
    state.rules.switching_stamina_cost = 0.0;
    state.rules.max_switches_per_turn = 3;
    state.rebuild_rope_counts();
    state
}

/// Arena game with uneven teams: 3 players for A, 2 for B.
/// Unequal budgets are the classic Colonel Blotto case where the weaker
/// side must randomize.
pub fn create_uneven_arena_game() -> GameState {
    let mut state = create_arena_game();
    state.players.pop();
    state.assignments.pop();
    state.rebuild_rope_counts();
    state
}
