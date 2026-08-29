use crate::game::{GameState, Team, RopeId};

/// Information model for partial observability
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum InformationModel {
    PerfectInformation,
    LocalInformation,
    DelayedInformation(usize), // Delay in turns
}

/// Observation that a strategy receives
#[derive(Clone, Debug)]
pub struct Observation {
    pub rope_positions: Vec<Option<i32>>, // None if not visible
    pub rope_forces: Vec<Option<f64>>,    // None if not visible
    pub opponent_allocations: Vec<Option<usize>>, // None if not visible
    pub team_energy: f64,
    pub time: usize,
}

impl Observation {
    /// Create a perfect information observation
    pub fn perfect(state: &GameState, team: Team) -> Self {
        Self {
            rope_positions: state.ropes.iter().map(|r| Some(r.position)).collect(),
            rope_forces: state.ropes.iter()
                .enumerate()
                .map(|(i, _)| {
                    let force_a = crate::game::physics::calculate_team_force(state, Team::A, i);
                    let force_b = crate::game::physics::calculate_team_force(state, Team::B, i);
                    Some(force_a - force_b)
                })
                .collect(),
            opponent_allocations: state.ropes.iter()
                .map(|r| {
                    let opponent_count = match team {
                        Team::A => r.team_b_count,
                        Team::B => r.team_a_count,
                    };
                    Some(opponent_count)
                })
                .collect(),
            team_energy: state.get_team_energy(team),
            time: state.time,
        }
    }
    
    /// Create a local information observation (only current rope)
    pub fn local(state: &GameState, team: Team, player_rope: RopeId) -> Self {
        let rope_count = state.ropes.len();
        let mut rope_positions = vec![None; rope_count];
        let mut rope_forces = vec![None; rope_count];
        let mut opponent_allocations = vec![None; rope_count];
        
        // Only observe current rope
        if player_rope < rope_count {
            rope_positions[player_rope] = Some(state.ropes[player_rope].position);
            
            let force_a = crate::game::physics::calculate_team_force(state, Team::A, player_rope);
            let force_b = crate::game::physics::calculate_team_force(state, Team::B, player_rope);
            rope_forces[player_rope] = Some(force_a - force_b);
            
            let opponent_count = match team {
                Team::A => state.ropes[player_rope].team_b_count,
                Team::B => state.ropes[player_rope].team_a_count,
            };
            opponent_allocations[player_rope] = Some(opponent_count);
        }
        
        Self {
            rope_positions,
            rope_forces,
            opponent_allocations,
            team_energy: state.get_team_energy(team),
            time: state.time,
        }
    }
    
    /// Create a delayed information observation
    pub fn delayed(state: &GameState, team: Team, delay: usize) -> Self {
        let effective_time = state.time.saturating_sub(delay);
        
        // This is a simplified version - in a real implementation,
        // you'd need to store historical observations
        if effective_time == state.time {
            Self::perfect(state, team)
        } else {
            // Return limited information for delayed observation
            Self {
                rope_positions: state.ropes.iter().map(|r| Some(r.position)).collect(),
                rope_forces: vec![None; state.ropes.len()], // No force information when delayed
                opponent_allocations: vec![None; state.ropes.len()], // No allocation info when delayed
                team_energy: state.get_team_energy(team),
                time: state.time,
            }
        }
    }
}

/// Information manager
pub struct InformationManager {
    model: InformationModel,
}

impl InformationManager {
    pub fn new(model: InformationModel) -> Self {
        Self { model }
    }
    
    /// Get observation for a team
    pub fn get_observation(&self, state: &GameState, team: Team, player_rope: RopeId) -> Observation {
        match self.model {
            InformationModel::PerfectInformation => Observation::perfect(state, team),
            InformationModel::LocalInformation => Observation::local(state, team, player_rope),
            InformationModel::DelayedInformation(delay) => Observation::delayed(state, team, delay),
        }
    }
    
    /// Check if information is perfect
    pub fn is_perfect(&self) -> bool {
        matches!(self.model, InformationModel::PerfectInformation)
    }
}