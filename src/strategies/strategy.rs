use crate::game::{GameState, Team};

#[derive(Clone, Debug)]
pub struct Move {
    pub player: usize,
    pub from: usize,
    pub to: usize,
}

#[derive(Clone, Debug)]
pub struct TeamAction {
    pub moves: Vec<Move>,
}

impl TeamAction {
    pub fn empty() -> Self {
        Self { moves: Vec::new() }
    }
}

#[derive(Clone, Debug)]
pub struct Allocation {
    pub counts: Vec<usize>,
}

impl Allocation {
    pub fn new(ropes: usize) -> Self {
        Self {
            counts: vec![0; ropes],
        }
    }
}

/// Strategy trait for different decision-making approaches
pub trait Strategy: Send + Sync {
    fn decide(&mut self, state: &GameState, team: Team) -> TeamAction;
    fn name(&self) -> &str;
}

/// Helper function to convert allocation to player moves
pub fn allocation_to_action(
    state: &GameState,
    team: Team,
    allocation: &[usize], // allocation[rope_id] = number of players
) -> TeamAction {
    let mut action = TeamAction::empty();
    
    // Group current players by rope
    let mut players_by_rope: Vec<Vec<usize>> = vec![Vec::new(); state.ropes.len()];
    
    for player in &state.players {
        if player.team == team {
            let rope = state.assignments[player.id];
            if rope < players_by_rope.len() {
                players_by_rope[rope].push(player.id);
            }
        }
    }
    
    // For each rope, ensure we have the desired number of players
    for target_rope in 0..allocation.len() {
        let desired = allocation[target_rope];
        let current = players_by_rope[target_rope].len();
        
        if desired > current {
            // Need more players on this rope
            let needed = desired - current;
            
            // Find players on other ropes
            for (source_rope, players) in players_by_rope.iter().enumerate() {
                if source_rope == target_rope {
                    continue;
                }
                
                for &player_id in players {
                    if action.moves.len() >= needed {
                        break;
                    }
                    
                    action.moves.push(Move {
                        player: player_id,
                        from: source_rope,
                        to: target_rope,
                    });
                }
                
                if action.moves.len() >= needed {
                    break;
                }
            }
        }
    }
    
    action
}