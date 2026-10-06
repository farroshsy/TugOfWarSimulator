use tug_of_war::experiments::{ExperimentRunner, ExperimentConfig, StrategyType, GameSize};
use tug_of_war::experiments::metrics::format_comparison_table;
use std::path::PathBuf;

fn main() {
    println!("=== Experiment Runner Test ===\n");
    
    // Test different strategy combinations
    let strategies = vec![
        ("Random vs Greedy", StrategyType::Random, StrategyType::Greedy, GameSize::Small),
        ("Greedy vs Reactive", StrategyType::Greedy, StrategyType::Reactive, GameSize::Small),
        ("Reactive vs FatigueAware", StrategyType::Reactive, StrategyType::FatigueAware, GameSize::Small),
        ("FatigueAware vs Random", StrategyType::FatigueAware, StrategyType::Random, GameSize::Small),
        // DP strategies on the tiny game (exact solvers are exponential in turns).
        // ShapleyDP plays both sides; DP and FatigueDP only support Team A.
        ("ShapleyDP vs Greedy", StrategyType::ShapleyDp, StrategyType::Greedy, GameSize::Tiny),
        ("Greedy vs ShapleyDP", StrategyType::Greedy, StrategyType::ShapleyDp, GameSize::Tiny),
        ("ShapleyDP vs Reactive", StrategyType::ShapleyDp, StrategyType::Reactive, GameSize::Tiny),
        ("Reactive vs ShapleyDP", StrategyType::Reactive, StrategyType::ShapleyDp, GameSize::Tiny),
        ("ShapleyDP vs Random", StrategyType::ShapleyDp, StrategyType::Random, GameSize::Tiny),
        ("DP vs ShapleyDP", StrategyType::Dp, StrategyType::ShapleyDp, GameSize::Tiny),
        ("FatigueDP vs ShapleyDP", StrategyType::FatigueDp, StrategyType::ShapleyDp, GameSize::Tiny),
        ("ApproxDP vs ShapleyDP", StrategyType::ApproximateDp, StrategyType::ShapleyDp, GameSize::Tiny),
    ];
    
    let mut all_results = Vec::new();
    
    for (name, strategy_a, strategy_b, game_size) in strategies {
        println!("Running: {} vs {}", strategy_a.name(), strategy_b.name());
        
        let config = ExperimentConfig {
            number_of_games: 100,
            random_seed: 42,
            team_a_strategy: strategy_a,
            team_b_strategy: strategy_b,
            game_size,
            strength_variation: 0.2,
        };
        
        let runner = ExperimentRunner::new(config);
        let results = runner.run();
        
        println!("{}", results);
        println!();
        
        all_results.push((name.to_string(), results));
    }
    
    // Generate comparison table
    println!("=== Strategy Comparison ===\n");
    match format_comparison_table(&all_results) {
        Ok(table) => println!("{}", table),
        Err(e) => println!("Error generating comparison table: {}", e),
    }
    
    // Export results to CSV
    let csv_path = PathBuf::from("experiment_results.csv");
    match tug_of_war::experiments::export::export_comparison_to_csv(&all_results, &csv_path) {
        Ok(_) => println!("\nResults exported to: {:?}", csv_path),
        Err(e) => println!("Error exporting to CSV: {}", e),
    }
    
    println!("\n=== Test completed successfully ===");
}