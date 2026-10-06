use crate::game::{GameState, Team, PlayerMode};
use crate::game::physics::{update_rope_positions, DefaultPhysicsModel, calculate_team_force, contest_outcomes};
use rand::Rng;
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

/// Execute one simulation step (deterministic physics only).
///
/// Panics if the rules use stochastic physics: use `step_with_rng` to
/// sample, or `step_outcomes` to enumerate outcomes with probabilities.
pub fn step(
    state: &mut GameState,
    action_a: &TeamAction,
    action_b: &TeamAction,
) {
    assert!(
        state.rules.contest_decisiveness.is_none(),
        "stochastic physics: use step_with_rng or step_outcomes"
    );
    if state.finished() {
        return;
    }
    apply_actions(state, action_a, action_b);
    
    // Calculate and apply rope movement
    let physics = DefaultPhysicsModel::new(
        state.rules.use_weight_threshold,
        state.rules.movement_scale,
    );
    update_rope_positions(state, &physics);
    
    end_turn(state);
}

/// Execute one step, sampling stochastic rope movement with `rng`.
/// Identical to `step` for deterministic physics.
pub fn step_with_rng(
    state: &mut GameState,
    action_a: &TeamAction,
    action_b: &TeamAction,
    rng: &mut impl Rng,
) {
    if state.rules.contest_decisiveness.is_none() {
        step(state, action_a, action_b);
        return;
    }
    let mut outcomes = step_outcomes(state, action_a, action_b);
    let r: f64 = rng.gen();
    let mut cumulative = 0.0;
    let last = outcomes.len() - 1;
    for (i, (p, _)) in outcomes.iter().enumerate() {
        cumulative += p;
        if r < cumulative || i == last {
            *state = outcomes.swap_remove(i).1;
            return;
        }
    }
}

/// All possible next states with their probabilities. One outcome with
/// probability 1 for deterministic physics; for stochastic physics, the
/// product of each rope's independent contest outcomes.
pub fn step_outcomes(
    state: &GameState,
    action_a: &TeamAction,
    action_b: &TeamAction,
) -> Vec<(f64, GameState)> {
    let Some(decisiveness) = state.rules.contest_decisiveness else {
        let mut next = state.clone();
        step(&mut next, action_a, action_b);
        return vec![(1.0, next)];
    };
    if state.finished() {
        return vec![(1.0, state.clone())];
    }
    let mut base = state.clone();
    apply_actions(&mut base, action_a, action_b);
    
    let mut outcomes = vec![(1.0, base.clone())];
    for rope in 0..base.ropes.len() {
        let force_a = calculate_team_force(&base, Team::A, rope);
        let force_b = calculate_team_force(&base, Team::B, rope);
        let moves = contest_outcomes(force_a, force_b, decisiveness);
        outcomes = outcomes
            .into_iter()
            .flat_map(|(p, s)| {
                moves.iter().map(move |&(movement, q)| {
                    let mut next = s.clone();
                    let r = &mut next.ropes[rope];
                    r.position = (r.position + movement).clamp(r.min_position, r.max_position);
                    (p * q, next)
                })
            })
            .collect();
    }
    for (_, next) in outcomes.iter_mut() {
        end_turn(next);
    }
    outcomes
}

/// Everything in a step before rope movement.
fn apply_actions(state: &mut GameState, action_a: &TeamAction, action_b: &TeamAction) {
    // Execute pending commands
    execute_pending_commands(state);
    
    // Apply strategic movements
    apply_moves(state, action_a, Team::A);
    apply_moves(state, action_b, Team::B);
    
    // Apply spy behavior
    apply_spy_behavior(state);
}

/// Everything in a step after rope movement.
fn end_turn(state: &mut GameState) {
    // Update fatigue
    update_fatigue(state);
    
    // Advance time
    state.time += 1;
}
#[cfg(test)]
mod tests {
    use super::*;
    use crate::game::create_noisy_arena_game;
    use crate::strategies::allocation_to_action;
    use rand::SeedableRng;

    fn split_actions(state: &GameState) -> (TeamAction, TeamAction) {
        // A all on rope 1, B split: both ropes are contested or pulled.
        (
            allocation_to_action(state, Team::A, &[0, 3]),
            allocation_to_action(state, Team::B, &[1, 2]),
        )
    }

    #[test]
    fn stochastic_outcomes_are_a_distribution() {
        let state = create_noisy_arena_game();
        let (a, b) = split_actions(&state);
        let outcomes = step_outcomes(&state, &a, &b);
        // Rope 0: only B pulls (certain). Rope 1: contested (2 outcomes).
        assert_eq!(outcomes.len(), 2);
        let total: f64 = outcomes.iter().map(|(p, _)| p).sum();
        assert!((total - 1.0).abs() < 1e-12);
        assert!(outcomes.iter().all(|(_, s)| s.time == 1));
    }

    #[test]
    fn sampled_step_matches_outcome_frequencies() {
        let state = create_noisy_arena_game();
        let (a, b) = split_actions(&state);
        let outcomes = step_outcomes(&state, &a, &b);
        let p_rope1_toward_a: f64 = outcomes
            .iter()
            .filter(|(_, s)| s.ropes[1].position < 0)
            .map(|(p, _)| p)
            .sum();
        let mut rng = rand::rngs::StdRng::seed_from_u64(3);
        let n = 4000;
        let hits = (0..n)
            .filter(|_| {
                let mut s = state.clone();
                step_with_rng(&mut s, &a, &b, &mut rng);
                s.ropes[1].position < 0
            })
            .count();
        assert!((hits as f64 / n as f64 - p_rope1_toward_a).abs() < 0.03);
    }

    #[test]
    #[should_panic(expected = "stochastic physics")]
    fn deterministic_step_rejects_stochastic_rules() {
        let mut state = create_noisy_arena_game();
        let (a, b) = split_actions(&state);
        step(&mut state, &a, &b);
    }
}
