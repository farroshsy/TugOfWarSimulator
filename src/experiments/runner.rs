use crate::game::{GameState, Team, create_arena_game, create_uneven_arena_game, create_small_game, create_default_game};
use rand::{Rng, SeedableRng};
use crate::strategies::Strategy;
use crate::game::simulation::{step, calculate_score};
use std::sync::{Arc, Mutex};
use std::thread;

/// Experiment configuration
#[derive(Clone)]
pub struct ExperimentConfig {
    pub number_of_games: usize,
    pub random_seed: u64,
    pub team_a_strategy: StrategyType,
    pub team_b_strategy: StrategyType,
    pub game_size: GameSize,
    /// Each player's strength is scaled by a seeded random factor in
    /// [1 - v, 1 + v] before the game starts. 0.0 = identical players.
    pub strength_variation: f64,
}

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum StrategyType {
    Random,
    Greedy,
    Reactive,
    FatigueAware,
    /// Sequential maximin DP. Plays Team A only (falls back to Greedy as B).
    Dp,
    /// Fatigue-aware DP with compressed energy. Team A only.
    FatigueDp,
    /// Limited-horizon DP. Plays either team.
    ApproximateDp,
    /// Simultaneous-move (Shapley) DP with mixed strategies. Plays either team.
    ShapleyDp,
    /// Shapley DP maximizing P(win) - P(loss) instead of score margin.
    ShapleyDpWin,
}

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum GameSize {
    /// 3v3 arena (repeated Blotto); small enough for exact DP strategies.
    Arena,
    /// 3v2 arena; uneven teams, small enough for exact DP strategies.
    UnevenArena,
    Small,
    Default,
}

/// Experiment results
#[derive(Clone, Debug)]
pub struct ExperimentResults {
    pub wins_a: usize,
    pub wins_b: usize,
    pub draws: usize,
    pub average_score_a: f64,
    pub average_score_b: f64,
    pub average_score_differential: f64,
    pub average_energy_a: f64,
    pub average_energy_b: f64,
    pub total_games: usize,
}

impl ExperimentResults {
    pub fn win_rate_a(&self) -> f64 {
        if self.total_games == 0 {
            return 0.0;
        }
        self.wins_a as f64 / self.total_games as f64
    }
    
    pub fn win_rate_b(&self) -> f64 {
        if self.total_games == 0 {
            return 0.0;
        }
        self.wins_b as f64 / self.total_games as f64
    }
    
    pub fn draw_rate(&self) -> f64 {
        if self.total_games == 0 {
            return 0.0;
        }
        self.draws as f64 / self.total_games as f64
    }
}

/// Experiment runner
pub struct ExperimentRunner {
    config: ExperimentConfig,
}

impl ExperimentRunner {
    pub fn new(config: ExperimentConfig) -> Self {
        Self { config }
    }
    
    /// Run the experiment
    pub fn run(&self) -> ExperimentResults {
        let results = Arc::new(Mutex::new(ExperimentResults {
            wins_a: 0,
            wins_b: 0,
            draws: 0,
            average_score_a: 0.0,
            average_score_b: 0.0,
            average_score_differential: 0.0,
            average_energy_a: 0.0,
            average_energy_b: 0.0,
            total_games: 0,
        }));
        
        let num_threads = num_cpus::get().min(self.config.number_of_games);
        let games_per_thread = (self.config.number_of_games + num_threads - 1) / num_threads;
        
        let mut handles = vec![];
        
        for thread_id in 0..num_threads {
            let config = self.config.clone();
            let results = Arc::clone(&results);
            let start_game = thread_id * games_per_thread;
            let end_game = (start_game + games_per_thread).min(self.config.number_of_games);
            
            let handle = thread::spawn(move || {
                for game_idx in start_game..end_game {
                    let seed = config.random_seed + game_idx as u64;
                    let mut state = match config.game_size {
                        GameSize::Arena => create_arena_game(),
                        GameSize::UnevenArena => create_uneven_arena_game(),
                        GameSize::Small => create_small_game(),
                        GameSize::Default => create_default_game(),
                    };
                    apply_strength_variation(&mut state, config.strength_variation, seed);
                    
                    let mut strategy_a = config.team_a_strategy.create(seed, &state);
                    let mut strategy_b = config.team_b_strategy.create(seed + 1, &state);
                    
                    // Run the game
                    while !state.finished() {
                        let action_a = strategy_a.decide(&state, Team::A);
                        let action_b = strategy_b.decide(&state, Team::B);
                        step(&mut state, &action_a, &action_b);
                    }
                    
                    // Collect results
                    let score = calculate_score(&state);
                    let energy_a = state.get_team_energy(Team::A);
                    let energy_b = state.get_team_energy(Team::B);
                    
                    let mut results = results.lock().unwrap();
                    
                    if score.team_a > score.team_b {
                        results.wins_a += 1;
                    } else if score.team_b > score.team_a {
                        results.wins_b += 1;
                    } else {
                        results.draws += 1;
                    }
                    
                    results.average_score_a += score.team_a as f64;
                    results.average_score_b += score.team_b as f64;
                    results.average_score_differential += (score.team_a - score.team_b) as f64;
                    results.average_energy_a += energy_a;
                    results.average_energy_b += energy_b;
                    results.total_games += 1;
                }
            });
            
            handles.push(handle);
        }
        
        // Wait for all threads to complete
        for handle in handles {
            handle.join().expect("Thread panicked");
        }
        
        // Calculate averages
        let mut results = results.lock().unwrap();
        if results.total_games > 0 {
            results.average_score_a /= results.total_games as f64;
            results.average_score_b /= results.total_games as f64;
            results.average_score_differential /= results.total_games as f64;
            results.average_energy_a /= results.total_games as f64;
            results.average_energy_b /= results.total_games as f64;
        }
        
        results.clone()
    }
}

/// Scale every player's strength by a random factor in [1 - variation, 1 + variation].
/// Deterministic for a given seed, so experiments stay reproducible.
fn apply_strength_variation(state: &mut GameState, variation: f64, seed: u64) {
    if variation <= 0.0 {
        return;
    }
    let mut rng = rand::rngs::StdRng::seed_from_u64(seed);
    for player in &mut state.players {
        player.strength *= 1.0 + rng.gen_range(-variation..=variation);
    }
}

impl StrategyType {
    pub fn create(&self, seed: u64, state: &GameState) -> Box<dyn Strategy> {
        let people_a = state.get_team_player_count(Team::A);
        let people_b = state.get_team_player_count(Team::B);
        let ropes = state.ropes.len();
        match self {
            StrategyType::Random => Box::new(crate::strategies::RandomStrategy::new(seed)),
            StrategyType::Greedy => Box::new(crate::strategies::GreedyStrategy::new()),
            StrategyType::Reactive => Box::new(crate::strategies::ReactiveStrategy::new()),
            StrategyType::FatigueAware => Box::new(crate::strategies::FatigueAwareStrategy::new()),
            StrategyType::Dp => Box::new(crate::algorithms::DpStrategy::for_teams(people_a, people_b, ropes)),
            StrategyType::FatigueDp => Box::new(crate::algorithms::FatigueDpStrategy::for_teams(people_a, people_b, ropes, 10)),
            StrategyType::ApproximateDp => Box::new(crate::algorithms::ApproximateDpStrategy::new(people_a, ropes, 3, 4)),
            StrategyType::ShapleyDp => Box::new(crate::algorithms::ShapleyDpStrategy::new(seed)),
            StrategyType::ShapleyDpWin => Box::new(crate::algorithms::ShapleyDpStrategy::with_objective(
                seed,
                crate::algorithms::Objective::WinProbability,
            )),
        }
    }
    
    pub fn name(&self) -> &str {
        match self {
            StrategyType::Random => "Random",
            StrategyType::Greedy => "Greedy",
            StrategyType::Reactive => "Reactive",
            StrategyType::FatigueAware => "FatigueAware",
            StrategyType::Dp => "DP",
            StrategyType::FatigueDp => "FatigueDP",
            StrategyType::ApproximateDp => "ApproxDP",
            StrategyType::ShapleyDp => "ShapleyDP",
            StrategyType::ShapleyDpWin => "ShapleyDP-Win",
        }
    }
}

impl GameSize {
    pub fn name(&self) -> &str {
        match self {
            GameSize::Arena => "Arena (3v3, 2 light ropes, 6 turns)",
            GameSize::UnevenArena => "Uneven Arena (3v2, 2 light ropes, 6 turns)",
            GameSize::Small => "Small (3v3, 2 ropes)",
            GameSize::Default => "Default (12v12, 3 ropes)",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn strengths(variation: f64, seed: u64) -> Vec<f64> {
        let mut state = create_small_game();
        apply_strength_variation(&mut state, variation, seed);
        state.players.iter().map(|p| p.strength).collect()
    }

    #[test]
    fn zero_variation_leaves_strengths_unchanged() {
        let base: Vec<f64> = create_small_game().players.iter().map(|p| p.strength).collect();
        assert_eq!(strengths(0.0, 7), base);
    }

    #[test]
    fn same_seed_gives_same_strengths() {
        assert_eq!(strengths(0.2, 7), strengths(0.2, 7));
    }

    #[test]
    fn different_seeds_give_different_strengths() {
        assert_ne!(strengths(0.2, 7), strengths(0.2, 8));
    }

    #[test]
    fn strengths_stay_within_variation_bounds() {
        let base = create_small_game().players[0].strength;
        for s in strengths(0.2, 7) {
            assert!(s >= base * 0.8 && s <= base * 1.2);
        }
    }
}
