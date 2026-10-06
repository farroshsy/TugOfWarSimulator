use eframe::egui;
use crate::game::{GameState, Team, create_default_game, create_small_game, create_arena_game, create_uneven_arena_game};
use crate::strategies::{Strategy, GreedyStrategy, ReactiveStrategy};
use crate::experiments::{StrategyType, GameSize};
use crate::game::simulation::{step, calculate_score};

const ALL_STRATEGIES: [StrategyType; 8] = [
    StrategyType::Random,
    StrategyType::Greedy,
    StrategyType::Reactive,
    StrategyType::FatigueAware,
    StrategyType::ApproximateDp,
    StrategyType::Dp,
    StrategyType::FatigueDp,
    StrategyType::ShapleyDp,
];

/// Exact DP solvers are exponential in game length; only offer them on the arena games.
fn is_exact_dp(strategy: StrategyType) -> bool {
    matches!(strategy, StrategyType::Dp | StrategyType::FatigueDp | StrategyType::ShapleyDp)
}

pub struct TugOfWarApp {
    state: GameState,
    running: bool,
    speed: f32,
    strategy_a: StrategyType,
    strategy_b: StrategyType,
    strategy_a_impl: Option<Box<dyn Strategy>>,
    strategy_b_impl: Option<Box<dyn Strategy>>,
    seed: u64,
    game_size: GameSize,
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
                    
                    let exact_size = matches!(self.game_size, GameSize::Arena | GameSize::UnevenArena);
                    
                    ui.label("Team A Strategy:");
                    for strategy in ALL_STRATEGIES {
                        ui.add_enabled_ui(exact_size || !is_exact_dp(strategy), |ui| {
                            ui.radio_value(&mut self.strategy_a, strategy, strategy.name())
                                .on_disabled_hover_text("Exact DP: select an Arena game and Reset");
                        });
                    }
                    
                    ui.label("Team B Strategy:");
                    for strategy in ALL_STRATEGIES {
                        ui.add_enabled_ui(exact_size || !is_exact_dp(strategy), |ui| {
                            ui.radio_value(&mut self.strategy_b, strategy, strategy.name())
                                .on_disabled_hover_text("Exact DP: select an Arena game and Reset");
                        });
                    }
                    ui.small("DP and FatigueDP plan for Team A only (Greedy as B).");
                    
                    ui.separator();
                    
                    // Game size selection
                    ui.heading("Game Configuration");
                    ui.separator();
                    
                    ui.label("Game Size:");
                    let exact_dp_selected = is_exact_dp(self.strategy_a) || is_exact_dp(self.strategy_b);
                    for size in [GameSize::Arena, GameSize::UnevenArena, GameSize::Small, GameSize::Default] {
                        let exact_ok = matches!(size, GameSize::Arena | GameSize::UnevenArena);
                        ui.add_enabled_ui(exact_ok || !exact_dp_selected, |ui| {
                            ui.radio_value(&mut self.game_size, size, size.name())
                                .on_disabled_hover_text("Exact DP strategies need an Arena game");
                        });
                    }
                    
                    ui.separator();
                    
                    // Seed display
                    ui.label(format!("Seed: {}", self.seed));
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
        
        step(&mut self.state, &action_a, &action_b);
    }
    
    fn reset_game(&mut self) {
        self.state = match self.game_size {
            GameSize::Arena => create_arena_game(),
            GameSize::UnevenArena => create_uneven_arena_game(),
            GameSize::Small => create_small_game(),
            GameSize::Default => create_default_game(),
        };
        self.running = false;
        self.strategy_a_impl = Some(self.strategy_a.create(self.seed, &self.state));
        self.strategy_b_impl = Some(self.strategy_b.create(self.seed + 1, &self.state));
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