use crate::game::{Player, Rope, GameRules, Team, PlayerId, RopeId};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct GameState {
    pub time: usize,
    pub players: Vec<Player>,
    pub ropes: Vec<Rope>,
    pub assignments: Vec<RopeId>, // player_id -> rope_id
    pub rules: GameRules,
    
    // Command queue: (execution_time, player_id, target_rope)
    pub pending_commands: Vec<(usize, PlayerId, RopeId)>,
    pub last_command_time: Option<usize>,
    
    // Random seed for reproducibility
    pub seed: u64,
}

impl GameState {
    pub fn new(
        players: Vec<Player>,
        ropes: Vec<Rope>,
        rules: GameRules,
        seed: u64,
    ) -> Self {
        let assignments = vec![0; players.len()];
        
        let mut state = Self {
            time: 0,
            players,
            ropes,
            assignments,
            rules,
            pending_commands: Vec::new(),
            last_command_time: None,
            seed,
        };
        
        state.rebuild_rope_counts();
        state
    }
    
    pub fn finished(&self) -> bool {
        self.time >= self.rules.duration
    }
    
    pub fn rebuild_rope_counts(&mut self) {
        for rope in &mut self.ropes {
            rope.team_a_count = 0;
            rope.team_b_count = 0;
        }
        
        for player in &self.players {
            let rope_id = self.assignments[player.id];
            match player.team {
                Team::A => {
                    if rope_id < self.ropes.len() {
                        self.ropes[rope_id].team_a_count += 1;
                    }
                }
                Team::B => {
                    if rope_id < self.ropes.len() {
                        self.ropes[rope_id].team_b_count += 1;
                    }
                }
            }
        }
    }
    
    pub fn get_team_players(&self, team: Team) -> Vec<&Player> {
        self.players.iter()
            .filter(|p| p.team == team)
            .collect()
    }
    
    pub fn get_team_player_count(&self, team: Team) -> usize {
        self.players.iter()
            .filter(|p| p.team == team)
            .count()
    }
    
    pub fn get_team_energy(&self, team: Team) -> f64 {
        self.players.iter()
            .filter(|p| p.team == team)
            .map(|p| p.stamina)
            .sum()
    }
    
    pub fn get_team_max_energy(&self, team: Team) -> f64 {
        self.players.iter()
            .filter(|p| p.team == team)
            .map(|p| p.max_stamina)
            .sum()
    }
}