use crate::game::{GameState, Team, RopeId};

/// Physics model trait for extensibility
pub trait PhysicsModel {
    fn calculate_movement(
        &self,
        rope: &crate::game::Rope,
        force_a: f64,
        force_b: f64,
    ) -> i32;
}

/// Default physics model with weight threshold
#[derive(Clone, Debug)]
pub struct DefaultPhysicsModel {
    pub use_weight_threshold: bool,
    pub movement_scale: f64,
}

impl DefaultPhysicsModel {
    pub fn new(use_weight_threshold: bool, movement_scale: f64) -> Self {
        Self {
            use_weight_threshold,
            movement_scale,
        }
    }
}

impl PhysicsModel for DefaultPhysicsModel {
    fn calculate_movement(
        &self,
        rope: &crate::game::Rope,
        force_a: f64,
        force_b: f64,
    ) -> i32 {
        let net_force = force_a - force_b;
        
        let movement = if self.use_weight_threshold {
            if net_force > rope.weight {
                self.movement_scale
            } else if net_force < -rope.weight {
                -self.movement_scale
            } else {
                0.0
            }
        } else {
            net_force.signum() * self.movement_scale
        };
        
        movement.round() as i32
    }
}

/// Calculate total force exerted by a team on a specific rope
pub fn calculate_team_force(
    state: &GameState,
    team: Team,
    rope_id: RopeId,
) -> f64 {
    state.players.iter()
        .filter(|p| p.team == team && state.assignments[p.id] == rope_id)
        .map(|p| p.effective_strength())
        .sum()
}

/// Calculate net force on a rope (positive = toward Team A)
pub fn calculate_net_force(
    state: &GameState,
    rope_id: RopeId,
) -> f64 {
    let force_a = calculate_team_force(state, Team::A, rope_id);
    let force_b = calculate_team_force(state, Team::B, rope_id);
    force_a - force_b
}

/// Update rope positions based on forces
pub fn update_rope_positions(state: &mut GameState, physics: &dyn PhysicsModel) {
    for i in 0..state.ropes.len() {
        let rope = state.ropes[i].clone();
        let force_a = calculate_team_force(state, Team::A, i);
        let force_b = calculate_team_force(state, Team::B, i);
        
        let movement = physics.calculate_movement(&rope, force_a, force_b);
        
        let new_position = rope.position as f64 + movement as f64;
        state.ropes[i].position = new_position
            .round()
            .clamp(rope.min_position as f64, rope.max_position as f64) as i32;
    }
}