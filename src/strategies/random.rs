use crate::game::{GameState, Team};
use crate::strategies::{Strategy, TeamAction, allocation_to_action};
use rand::Rng;
use rand::SeedableRng;

pub struct RandomStrategy {
    rng: rand::rngs::StdRng,
}

impl RandomStrategy {
    pub fn new(seed: u64) -> Self {
        Self {
            rng: rand::rngs::StdRng::seed_from_u64(seed),
        }
    }
    
    /// Generate a random allocation of players across ropes
    fn random_allocation(&mut self, num_players: usize, num_ropes: usize) -> Vec<usize> {
        if num_ropes == 0 {
            return vec![];
        }
        
        let mut allocation = vec![0; num_ropes];
        let mut remaining = num_players;
        
        for i in 0..num_ropes - 1 {
            if remaining == 0 {
                break;
            }
            let count = self.rng.gen_range(0..=remaining);
            allocation[i] = count;
            remaining -= count;
        }
        
        allocation[num_ropes - 1] = remaining;
        allocation
    }
}

impl Strategy for RandomStrategy {
    fn decide(&mut self, state: &GameState, team: Team) -> TeamAction {
        let num_players = state.get_team_player_count(team);
        let num_ropes = state.ropes.len();
        
        let allocation = self.random_allocation(num_players, num_ropes);
        allocation_to_action(state, team, &allocation)
    }
    
    fn name(&self) -> &str {
        "Random"
    }
}