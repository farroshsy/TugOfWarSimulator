use crate::game::{GameState, Team, PlayerId, RopeId};

/// Commander mechanics with communication delay
pub struct CommanderSystem {
    command_cooldown: usize,
    command_delay: usize,
    enabled: bool,
}

impl CommanderSystem {
    pub fn new(cooldown: usize, delay: usize, enabled: bool) -> Self {
        Self {
            command_cooldown: cooldown,
            command_delay: delay,
            enabled,
        }
    }
    
    /// Check if commander can issue a command
    pub fn can_issue_command(&self, state: &GameState, team: Team) -> bool {
        if !self.enabled {
            return false;
        }
        
        // Check if team has a commander
        let has_commander = state.players.iter()
            .any(|p| p.team == team && p.role == crate::game::Role::Commander);
        
        if !has_commander {
            return false;
        }
        
        // Check cooldown
        match state.last_command_time {
            None => true,
            Some(last) => state.time >= last + self.command_cooldown,
        }
    }
    
    /// Issue a command with delay
    pub fn issue_command(&self, state: &mut GameState, team: Team, player: PlayerId, rope: RopeId) {
        if !self.can_issue_command(state, team) {
            return;
        }
        
        let execute_at = state.time + self.command_delay;
        state.pending_commands.push((execute_at, player, rope));
        state.last_command_time = Some(state.time);
    }
    
    /// Execute pending commands
    pub fn execute_pending_commands(state: &mut GameState) {
        let current_time = state.time;
        let mut remaining = Vec::new();
        
        for command in state.pending_commands.drain(..) {
            let (execute_at, player, rope) = command;
            
            if execute_at <= current_time {
                state.assignments[player] = rope;
                if let Some(p) = state.players.get_mut(player) {
                    p.current_rope = rope;
                }
            } else {
                remaining.push(command);
            }
        }
        
        state.pending_commands = remaining;
        state.rebuild_rope_counts();
    }
    
    /// Get command queue status
    pub fn command_queue_status(&self, state: &GameState) -> (usize, usize) {
        let pending = state.pending_commands.len();
        let next_execution = state.pending_commands
            .iter()
            .map(|(time, _, _)| *time)
            .min()
            .unwrap_or(usize::MAX);
        
        (pending, next_execution)
    }
}