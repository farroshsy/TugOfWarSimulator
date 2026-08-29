use crate::game::{GameState, Team, PlayerMode};
use crate::game::physics::{update_rope_positions, DefaultPhysicsModel};
use crate::strategies::TeamAction;

#[derive(Clone, Debug)]
pub struct Score {
    pub team_a: i32,
    pub team_b: i32,
}

impl Score {
    pub fn differential(&self) -> i32 {
        self.team_a - self.team_b
    }
    
    pub fn winner(&self) -> Option<Team> {
        if self.team_a > self.team_b {
            Some(Team::A)
        } else if self.team_b > self.team_a {
            Some(Team::B)
        } else {
            None
        }
    }
}

/// Calculate final score based on rope positions
pub fn calculate_score(state: &GameState) -> Score {
    let mut team_a = 0;
    let mut team_b = 0;
    
    for rope in &state.ropes {
        if rope.position < 0 {
            team_a += rope.value;
        } else if rope.position > 0 {
            team_b += rope.value;
        }
        // Position == 0: no points for either team
    }
    
    Score { team_a, team_b }
}

/// Update player fatigue/stamina
pub fn update_fatigue(state: &mut GameState) {
    if !state.rules.fatigue_enabled {
        return;
    }
    
    for player in &mut state.players {
        match player.mode {
            PlayerMode::Pull => {
                player.stamina -= player.fatigue_rate * state.rules.dt;
                player.stamina = player.stamina.max(0.0);
            }
            PlayerMode::Rest => {
                player.stamina += player.recovery_rate * state.rules.dt;
                player.stamina = player.stamina.min(player.max_stamina);
            }
            PlayerMode::Guard => {
                // Guarding uses less energy than pulling
                player.stamina -= player.fatigue_rate * state.rules.dt * 0.5;
                player.stamina = player.stamina.max(0.0);
            }
            PlayerMode::Observe => {
                // Observing recovers some energy
                player.stamina += player.recovery_rate * state.rules.dt * 0.5;
                player.stamina = player.stamina.min(player.max_stamina);
            }
        }
    }
}

/// Apply player moves (switching ropes)
pub fn apply_moves(
    state: &mut GameState,
    action: &TeamAction,
    team: Team,
) {
    if !state.rules.switching_enabled {
        return;
    }
    
    let mut switches = 0;
    
    for mv in &action.moves {
        if switches >= state.rules.max_switches_per_turn {
            break;
        }
        
        let player = &state.players[mv.player];
        
        if player.team != team {
            continue;
        }
        
        if state.assignments[mv.player] != mv.from {
            continue;
        }
        
        if mv.from == mv.to {
            continue;
        }
        
        // Apply the move
        state.assignments[mv.player] = mv.to;
        state.players[mv.player].current_rope = mv.to;
        
        // Apply switching cost
        state.players[mv.player].stamina -= state.rules.switching_stamina_cost;
        state.players[mv.player].stamina = state.players[mv.player].stamina.max(0.0);
        
        switches += 1;
    }
    
    state.rebuild_rope_counts();
}

/// Execute pending commander commands
pub fn execute_pending_commands(state: &mut GameState) {
    if !state.rules.commander_enabled {
        return;
    }
    
    let current_time = state.time;
    let mut remaining = Vec::new();
    
    for command in state.pending_commands.drain(..) {
        let (execute_at, player, rope) = command;
        
        if execute_at <= current_time {
            state.assignments[player] = rope;
            state.players[player].current_rope = rope;
        } else {
            remaining.push(command);
        }
    }
    
    state.pending_commands = remaining;
    state.rebuild_rope_counts();
}

/// Apply spy behavior (simplified deterministic version)
pub fn apply_spy_behavior(state: &mut GameState) {
    if !state.rules.spies_enabled {
        return;
    }
    
    for player in state.players.clone().iter() {
        if player.role != crate::game::Role::Spy {
            continue;
        }
        
        // Simple spy behavior: move to rope that helps opponent
        let current = state.assignments[player.id];
        let mut best_enemy_rope = 0;
        let mut best_value = -1;
        
        for rope in &state.ropes {
            let is_bad_for_own_team = match player.team {
                Team::A => rope.position > 0,
                Team::B => rope.position < 0,
            };
            
            if is_bad_for_own_team && rope.value > best_value {
                best_value = rope.value;
                best_enemy_rope = rope.id;
            }
        }
        
        if current != best_enemy_rope {
            state.assignments[player.id] = best_enemy_rope;
            state.players[player.id].current_rope = best_enemy_rope;
        }
    }
    
    state.rebuild_rope_counts();
}

/// Execute one simulation step
pub fn step(
    state: &mut GameState,
    action_a: &TeamAction,
    action_b: &TeamAction,
) {
    if state.finished() {
        return;
    }
    
    // Execute pending commands
    execute_pending_commands(state);
    
    // Apply strategic movements
    apply_moves(state, action_a, Team::A);
    apply_moves(state, action_b, Team::B);
    
    // Apply spy behavior
    apply_spy_behavior(state);
    
    // Calculate and apply rope movement
    let physics = DefaultPhysicsModel::new(
        state.rules.use_weight_threshold,
        state.rules.movement_scale,
    );
    update_rope_positions(state, &physics);
    
    // Update fatigue
    update_fatigue(state);
    
    // Advance time
    state.time += 1;
}