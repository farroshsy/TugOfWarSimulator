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
        // Positions: negative = Team A territory, positive = Team B territory.
        // So a stronger Team A must move the rope in the negative direction.
        let net_force = force_b - force_a;

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

/// Possible movements of one rope this turn under stochastic (Tullock
/// contest) physics, as (movement, probability). Positive = toward Team B
/// territory, matching `calculate_movement`.
pub fn contest_outcomes(force_a: f64, force_b: f64, decisiveness: f64) -> Vec<(i32, f64)> {
    let a = force_a.max(0.0).powf(decisiveness);
    let b = force_b.max(0.0).powf(decisiveness);
    if a + b <= 0.0 {
        return vec![(0, 1.0)];
    }
    let p_toward_a = a / (a + b);
    [(-1, p_toward_a), (1, 1.0 - p_toward_a)]
        .into_iter()
        .filter(|&(_, p)| p > 0.0)
        .collect()
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::game::Rope;

    fn rope(weight: f64) -> Rope {
        Rope::new(0, 0, -10, 10, weight, 1)
    }

    #[test]
    fn stronger_team_a_pulls_rope_into_a_territory() {
        let physics = DefaultPhysicsModel::new(true, 1.0);
        assert_eq!(physics.calculate_movement(&rope(1.0), 10.0, 5.0), -1);
    }

    #[test]
    fn stronger_team_b_pulls_rope_into_b_territory() {
        let physics = DefaultPhysicsModel::new(true, 1.0);
        assert_eq!(physics.calculate_movement(&rope(1.0), 5.0, 10.0), 1);
    }

    #[test]
    fn contest_probabilities_follow_force_ratio() {
        // r = 1: probability proportional to force.
        let outcomes = contest_outcomes(30.0, 10.0, 1.0);
        assert_eq!(outcomes, vec![(-1, 0.75), (1, 0.25)]);
        // Unopposed team always wins the pull.
        assert_eq!(contest_outcomes(5.0, 0.0, 2.0), vec![(-1, 1.0)]);
        // Nobody pulling: rope stays.
        assert_eq!(contest_outcomes(0.0, 0.0, 2.0), vec![(0, 1.0)]);
    }

    #[test]
    fn net_force_within_weight_does_not_move_rope() {
        let physics = DefaultPhysicsModel::new(true, 1.0);
        assert_eq!(physics.calculate_movement(&rope(3.0), 10.0, 8.0), 0);
    }
}
