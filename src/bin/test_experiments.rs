use tug_of_war::experiments::{ExperimentRunner, ExperimentConfig, StrategyType, GameSize};
use tug_of_war::experiments::metrics::format_comparison_table;
use std::path::PathBuf;

fn main() {
    println!("=== Experiment Runner Test ===\n");
    
    // Test different strategy combinations
    let strategies = vec![
        ("Random vs Greedy", StrategyType::Random, StrategyType::Greedy),
        ("Greedy vs Reactive", StrategyType::Greedy, StrategyType::Reactive),
        ("Reactive vs FatigueAware", StrategyType::Reactive, StrategyType::FatigueAware),
        ("FatigueAware vs Random", StrategyType::FatigueAware, StrategyType::Random),
    ];
    
    let mut all_results = Vec::new();
    
    for (name, strategy_a, strategy_b) in strategies {
        println!("Running: {} vs {}", strategy_a.name(), strategy_b.name());
        
        let config = ExperimentConfig {
            number_of_games: 100,
            random_seed: 42,
            team_a_strategy: strategy_a,
            team_b_strategy: strategy_b,
            game_size: GameSize::Small, // Use small game for faster testing
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