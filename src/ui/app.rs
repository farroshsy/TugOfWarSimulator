use eframe::egui;
use crate::game::{
    GameState, Team, create_default_game, create_small_game, create_arena_game, create_uneven_arena_game,
    create_noisy_arena_game, create_noisy_uneven_arena_game,
};
use crate::strategies::{Strategy, GreedyStrategy, ReactiveStrategy};
use crate::experiments::{StrategyType, GameSize};
use crate::algorithms::ShapleyDpSolver;
use crate::game::simulation::{step_with_rng, calculate_score};
use rand::SeedableRng;

const ALL_STRATEGIES: [StrategyType; 10] = [
    StrategyType::Random,
    StrategyType::Greedy,
    StrategyType::Reactive,
    StrategyType::FatigueAware,
    StrategyType::ApproximateDp,
    StrategyType::SmMcts,
    StrategyType::Dp,
    StrategyType::FatigueDp,
    StrategyType::ShapleyDp,
    StrategyType::ShapleyDpWin,
];

/// Exact DP solvers are exponential in game length; only offer them on the arena games.
fn supports_exact_dp(size: GameSize) -> bool {
    matches!(
        size,
        GameSize::Arena | GameSize::UnevenArena | GameSize::NoisyArena | GameSize::NoisyUnevenArena
    )
}

fn is_stochastic(size: GameSize) -> bool {
    matches!(size, GameSize::NoisyArena | GameSize::NoisyUnevenArena)
}

/// Rows for the DP Inspector: (allocation, probability) for a team's
/// equilibrium mixed strategy at this state, most likely first, zeros dropped.
fn inspector_rows(solver: &mut ShapleyDpSolver, state: &GameState, team: Team) -> Vec<(Vec<usize>, f64)> {
    let stage = solver.stage(state).clone();
    let probabilities = match team {
        Team::A => stage.row_strategy,
        Team::B => stage.col_strategy,
    };
    let mut rows: Vec<(Vec<usize>, f64)> = solver
        .allocations(team)
        .iter()
        .zip(probabilities)
        .filter(|(_, p)| *p > 1e-6)
        .map(|(allocation, p)| (allocation.counts.clone(), p))
        .collect();
    rows.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
    rows
}

fn is_exact_dp(strategy: StrategyType) -> bool {
    matches!(strategy, StrategyType::Dp | StrategyType::FatigueDp | StrategyType::ShapleyDp | StrategyType::ShapleyDpWin)
}

/// Whether a strategy can run on a game size without freezing the UI.
/// Exact DP is exponential in game length; SM-MCTS takes ~1s per turn on
/// the default game (1000 playouts of up to 60 turns with 24 players).
fn strategy_allowed(strategy: StrategyType, size: GameSize) -> bool {
    if matches!(strategy, StrategyType::Dp | StrategyType::FatigueDp) && is_stochastic(size) {
        // These solvers assume deterministic physics.
        false
    } else if is_exact_dp(strategy) {
        supports_exact_dp(size)
    } else if strategy == StrategyType::SmMcts {
        size != GameSize::Default
    } else {
        true
    }
}

const SLOW_STRATEGY_HINT: &str = "Too slow for this game size: pick a smaller game and Reset";

pub struct TugOfWarApp {
    state: GameState,
    running: bool,
    speed: f32,
    strategy_a: StrategyType,
    strategy_b: StrategyType,
    strategy_a_impl: Option<Box<dyn Strategy>>,
    strategy_b_impl: Option<Box<dyn Strategy>>,
    seed: u64,
    // Selected size; applied on Reset. `state_size` is what `state` was built with.
    game_size: GameSize,
    state_size: GameSize,
    show_inspector: bool,
    // Built on demand for the current game; cleared on reset.
    inspector_solver: Option<ShapleyDpSolver>,
    // Samples stochastic rope movement; separate from strategies' RNGs.
    physics_rng: rand::rngs::StdRng,
}

impl Default for TugOfWarApp {
    fn default() -> Self {
        let state = create_default_game();
        Self {
            state,
            running: false,
            speed: 1.0,
            strategy_a: StrategyType::Greedy,
            strategy_b: StrategyType::Reactive,
            strategy_a_impl: Some(Box::new(GreedyStrategy::new())),
            strategy_b_impl: Some(Box::new(ReactiveStrategy::new())),
            seed: 42,
            game_size: GameSize::Default,
            state_size: GameSize::Default,
            show_inspector: false,
            inspector_solver: None,
            physics_rng: rand::rngs::StdRng::seed_from_u64(42),
        }
    }
}

impl eframe::App for TugOfWarApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Top menu bar
        egui::TopBottomPanel::top("top_bar").show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.heading("Tug-of-War DP Simulator");
                ui.separator();
                ui.label(format!("Time: {}/{}", self.state.time, self.state.rules.duration));
                
                let score = calculate_score(&self.state);
                ui.separator();
                ui.label(format!("Team A Score: {}", score.team_a));
                ui.label(format!("Team B Score: {}", score.team_b));
                
                ui.separator();
                let energy_a = self.state.get_team_energy(Team::A);
                let energy_b = self.state.get_team_energy(Team::B);
                ui.label(format!("Team A Energy: {:.0}%", energy_a / self.state.get_team_max_energy(Team::A) * 100.0));
                ui.label(format!("Team B Energy: {:.0}%", energy_b / self.state.get_team_max_energy(Team::B) * 100.0));
            });
        });
        
        // Main content area
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.horizontal(|ui| {
                // Left panel: Arena visualization
                ui.vertical(|ui| {
                    ui.heading("Arena");
                    ui.separator();
                    self.draw_arena(ui);
                });
                
                ui.separator();
                
                // Right panel: Controls and info
                ui.vertical(|ui| {
                    // Game controls
                    ui.heading("Controls");
                    ui.separator();
                    ui.horizontal(|ui| {
                        if ui.button(if self.running { "Pause" } else { "Start" }).clicked() {
                            self.running = !self.running;
                        }
                        
                        if ui.button("Step").clicked() {
                            self.step_game();
                        }
                        
                        if ui.button("Reset").clicked() {
                            self.reset_game();
                        }
                    });
                    
                    ui.separator();
                    
                    // Speed control
                    ui.label("Speed:");
                    ui.add(egui::Slider::new(&mut self.speed, 0.25..=8.0).step_by(0.25).show_value(true));
                    
                    ui.separator();
                    
                    // Strategy selection
                    ui.heading("Strategies");
                    ui.separator();
                    
                    // Strategies act on the live state, so gate on the size it was built with.
                    ui.label("Team A Strategy:");
                    for strategy in ALL_STRATEGIES {
                        ui.add_enabled_ui(strategy_allowed(strategy, self.state_size), |ui| {
                            ui.radio_value(&mut self.strategy_a, strategy, strategy.name())
                                .on_disabled_hover_text(SLOW_STRATEGY_HINT);
                        });
                    }
                    
                    ui.label("Team B Strategy:");
                    for strategy in ALL_STRATEGIES {
                        ui.add_enabled_ui(strategy_allowed(strategy, self.state_size), |ui| {
                            ui.radio_value(&mut self.strategy_b, strategy, strategy.name())
                                .on_disabled_hover_text(SLOW_STRATEGY_HINT);
                        });
                    }
                    ui.small("DP and FatigueDP plan for Team A only (Greedy as B).");
                    
                    ui.separator();
                    
                    // Game size selection
                    ui.heading("Game Configuration");
                    ui.separator();
                    
                    ui.label("Game Size:");

                    for size in [
                        GameSize::Arena,
                        GameSize::UnevenArena,
                        GameSize::NoisyArena,
                        GameSize::NoisyUnevenArena,
                        GameSize::Small,
                        GameSize::Default,
                    ] {
                        let ok = strategy_allowed(self.strategy_a, size) && strategy_allowed(self.strategy_b, size);
                        ui.add_enabled_ui(ok, |ui| {
                            ui.radio_value(&mut self.game_size, size, size.name())
                                .on_disabled_hover_text("A selected strategy is too slow for this game size");
                        });
                    }
                    
                    ui.separator();
                    
                    // Seed display
                    ui.label(format!("Seed: {}", self.seed));
                    
                    ui.separator();
                    self.draw_inspector(ui);
                });
            });
        });
        
        // Run simulation if running
        if self.running && !self.state.finished() {
            // Run multiple steps based on speed
            let steps = (self.speed * 2.0) as usize;
            for _ in 0..steps.max(1) {
                if self.state.finished() {
                    break;
                }
                self.step_game();
            }
            
            // Request repaint for smooth animation
            ctx.request_repaint();
        }
    }
}

impl TugOfWarApp {
    fn step_game(&mut self) {
        if self.state.finished() {
            self.running = false;
            return;
        }
        
        // Recreate strategies if needed
        if self.strategy_a_impl.is_none() || self.strategy_a_impl.as_ref().unwrap().name() != self.strategy_a.name() {
            self.strategy_a_impl = Some(self.strategy_a.create(self.seed, &self.state));
        }
        if self.strategy_b_impl.is_none() || self.strategy_b_impl.as_ref().unwrap().name() != self.strategy_b.name() {
            self.strategy_b_impl = Some(self.strategy_b.create(self.seed + 1, &self.state));
        }
        
        let action_a = self.strategy_a_impl.as_mut().unwrap().decide(&self.state, Team::A);
        let action_b = self.strategy_b_impl.as_mut().unwrap().decide(&self.state, Team::B);
        
        step_with_rng(&mut self.state, &action_a, &action_b, &mut self.physics_rng);
    }
    
    fn reset_game(&mut self) {
        self.inspector_solver = None;
        self.state_size = self.game_size;
        self.state = match self.game_size {
            GameSize::Arena => create_arena_game(),
            GameSize::UnevenArena => create_uneven_arena_game(),
            GameSize::NoisyArena => create_noisy_arena_game(),
            GameSize::NoisyUnevenArena => create_noisy_uneven_arena_game(),
            GameSize::Small => create_small_game(),
            GameSize::Default => create_default_game(),
        };
        self.running = false;
        self.strategy_a_impl = Some(self.strategy_a.create(self.seed, &self.state));
        self.strategy_b_impl = Some(self.strategy_b.create(self.seed + 1, &self.state));
    }
    
    fn draw_inspector(&mut self, ui: &mut egui::Ui) {
        ui.heading("DP Inspector");
        let solvable = supports_exact_dp(self.state_size);
        ui.add_enabled(solvable, egui::Checkbox::new(&mut self.show_inspector, "Show Shapley DP solution"))
            .on_disabled_hover_text("Select an Arena game and Reset");
        if !self.show_inspector || !solvable {
            return;
        }
        if self.state.finished() {
            ui.label("Game over.");
            return;
        }

        let state = &self.state;
        let solver = self.inspector_solver.get_or_insert_with(|| ShapleyDpSolver::new(state));
        let value = solver.solve(state);
        let rows_a = inspector_rows(solver, state, Team::A);
        let rows_b = inspector_rows(solver, state, Team::B);
        let states = solver.cache_size();

        ui.label(format!("t = {}, rope positions = {:?}", state.time,
            state.ropes.iter().map(|r| r.position).collect::<Vec<_>>()));
        ui.label(format!("Game value (A's expected final margin): {:+.3}", value));
        ui.label(format!("States solved: {}", states));
        for (team, rows) in [(Team::A, rows_a), (Team::B, rows_b)] {
            ui.label(format!("Team {:?} optimal mix (players per rope):", team));
            for (allocation, p) in rows {
                ui.monospace(format!("  {:?}  {:>5.1}%", allocation, p * 100.0));
            }
        }
    }
    
    fn draw_arena(&mut self, ui: &mut egui::Ui) {
        ui.label("TEAM A");
        ui.separator();
        
        for rope in &self.state.ropes {
            ui.horizontal(|ui| {
                ui.label(format!("Rope {} (v={}, w={}):", rope.id, rope.value, rope.weight));
                
                // Draw rope position visualization
                let available_width = ui.available_width() - 200.0;
                let center = available_width / 2.0;
                let position_range = self.state.rules.position_range as f32;
                let rope_pos = rope.position as f32;
                
                // Calculate position in pixels
                let _pixel_pos = center + (rope_pos / position_range) * (center - 20.0);
                
                // Draw rope position with simple text
                let position_text = if rope.position < 0 {
                    format!("{} units in A territory", rope.position.abs())
                } else if rope.position > 0 {
                    format!("{} units in B territory", rope.position)
                } else {
                    "at center".to_string()
                };
                
                ui.label(position_text);
                
                // Simple bar visualization
                let bar_width = (rope.position.abs() as f32 / position_range) * (center - 20.0);
                let bar_color = if rope.position < 0 {
                    egui::Color32::BLUE
                } else if rope.position > 0 {
                    egui::Color32::RED
                } else {
                    egui::Color32::GRAY
                };
                
                ui.add_sized([200.0, 20.0], egui::ProgressBar::new(bar_width / (center - 20.0)).fill(bar_color));
                
                // Show player counts
                ui.label(format!("A: {} | B: {}", rope.team_a_count, rope.team_b_count));
            });
        }
        
        ui.separator();
        ui.label("TEAM B");
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn slow_strategies_are_gated_by_game_size() {
        assert!(strategy_allowed(StrategyType::ShapleyDp, GameSize::Arena));
        assert!(!strategy_allowed(StrategyType::ShapleyDp, GameSize::Small));
        assert!(strategy_allowed(StrategyType::SmMcts, GameSize::Small));
        assert!(!strategy_allowed(StrategyType::SmMcts, GameSize::Default));
        assert!(strategy_allowed(StrategyType::Greedy, GameSize::Default));
        assert!(strategy_allowed(StrategyType::ShapleyDp, GameSize::NoisyArena));
        assert!(!strategy_allowed(StrategyType::Dp, GameSize::NoisyArena));
    }

    #[test]
    fn inspector_rows_are_a_sorted_distribution() {
        let state = create_uneven_arena_game();
        let mut solver = ShapleyDpSolver::new(&state);
        for team in [Team::A, Team::B] {
            let rows = inspector_rows(&mut solver, &state, team);
            let total: f64 = rows.iter().map(|(_, p)| p).sum();
            assert!((total - 1.0).abs() < 1e-6);
            assert!(rows.windows(2).all(|w| w[0].1 >= w[1].1));
            let players = state.get_team_player_count(team);
            assert!(rows.iter().all(|(a, _)| a.iter().sum::<usize>() == players));
        }
    }
}
