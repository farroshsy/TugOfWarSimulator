# Experiments Guide

## Overview

This document explains how to use the experiment framework to run reproducible experiments, compare strategies, and analyze results.

## Experiment Configuration

### Basic Configuration Structure
```rust
ExperimentConfig {
    number_of_games: usize,
    random_seed: u64,
    team_a_strategy: StrategyType,
    team_b_strategy: StrategyType,
    game_size: GameSize,
    strength_variation: f64, // per-player strength scaled by seeded factor in [1-v, 1+v]; 0.0 = identical players
}
```

### Available Strategies
- `Random`: Random player allocation
- `Greedy`: Always attack highest-value rope
- `Reactive`: Consider rope position and opponent allocation
- `FatigueAware`: Conserve energy early, commit late
- `DP`: Exact minimax DP (small games only)
- `FatigueDP`: Fatigue-aware DP with compressed energy
- `ApproxDP`: Limited horizon beam search

### Game Sizes
- `Small`: 3v3, 2 ropes, 10 turns (for DP testing)
- `Default`: 12v12, 3 ropes, 60 turns (full simulation)

## Running Experiments

### Command Line
```bash
cargo run --bin test_experiments
```

### Programmatic Usage
```rust
use tug_of_war::experiments::{ExperimentRunner, ExperimentConfig, StrategyType, GameSize};

let config = ExperimentConfig {
    number_of_games: 1000,
    random_seed: 42,
    team_a_strategy: StrategyType::Greedy,
    team_b_strategy: StrategyType::Reactive,
    game_size: GameSize::Default,
    strength_variation: 0.2,
};

let runner = ExperimentRunner::new(config);
let results = runner.run();
println!("{}", results);
```

## Experiment Results

### Result Structure
```rust
ExperimentResults {
    wins_a: usize,
    wins_b: usize,
    draws: usize,
    average_score_a: f64,
    average_score_b: f64,
    average_score_differential: f64,
    average_energy_a: f64,
    average_energy_b: f64,
    total_games: usize,
}
```

### Derived Metrics
```text
win_rate_a = wins_a / total_games
win_rate_b = wins_b / total_games
draw_rate = draws / total_games
```

## Strategy Comparison

### Running Multiple Comparisons
```rust
let strategies = vec![
    ("Random vs Greedy", StrategyType::Random, StrategyType::Greedy),
    ("Greedy vs Reactive", StrategyType::Greedy, StrategyType::Reactive),
    ("Reactive vs FatigueAware", StrategyType::Reactive, StrategyType::FatigueAware),
];

let mut all_results = Vec::new();

for (name, strategy_a, strategy_b) in strategies {
    let config = ExperimentConfig { /* ... */ };
    let runner = ExperimentRunner::new(config);
    let results = runner.run();
    all_results.push((name.to_string(), results));
}
```

### Comparison Table
```rust
use tug_of_war::experiments::format_comparison_table;

let table = format_comparison_table(&all_results)?;
println!("{}", table);
```

Output:
```
Strategy Comparison
====================
Strategy                 Wins   Losses    Draws Win Rate  Avg Score
--------                 ----   ------    ----- --------  ---------
Random vs Greedy          100        0        0   100.0       1.00
Greedy vs Reactive          50        50        0    50.0       0.00
```

## Export Results

### CSV Export
```rust
use tug_of_war::experiments::export_comparison_to_csv;
use std::path::PathBuf;

let csv_path = PathBuf::from("strategy_comparison.csv");
export_comparison_to_csv(&all_results, &csv_path)?;
```

### CSV Format
```csv
strategy,wins,losses,draws,win_rate,average_score_differential
Random vs Greedy,100,0,0,1.0000,1.00
Greedy vs Reactive,50,50,0,0.5000,0.00
```

## Performance Considerations

### Multi-Threading
The experiment runner uses multi-threading for scalability:
```text
num_threads = min(num_cpus, number_of_games)
games_per_thread = (number_of_games + num_threads - 1) / num_threads
```

### Memory Usage
Each game state is cloned independently per thread, so memory scales with thread count.

### Estimating Experiment Duration
```text
Small game (3v3, 10 turns): ~1 second per 100 games
Default game (12v12, 60 turns): ~10 seconds per 100 games
10,000 games: ~15-20 minutes
```

## Statistical Analysis

### Confidence Intervals
```rust
use tug_of_war::experiments::calculate_win_rate_confidence_interval;

let (lower, upper) = calculate_win_rate_confidence_interval(
    results.wins_a, 
    results.total_games, 
    0.95  // 95% confidence
);
```

### Sample Size Planning
For detecting a 5% win rate difference with 95% confidence:
```text
n ≈ 1,000 games
```

For detecting a 1% difference:
```text
n ≈ 25,000 games
```

## Recommended Experiments

### 1. Ground Truth Validation
```text
Configuration: Small game (3v3, 2 ropes, 10 turns)
Strategies: DP vs Greedy vs Reactive vs Random
Games: 100
Purpose: Validate DP implementation
```

### 2. Strategy Hierarchy
```text
Configuration: Small game
Strategies: Random → Greedy → Reactive → FatigueAware → DP
Games: 500 each
Purpose: Measure improvement curve
```

### 3. Scaling Analysis
```text
Configurations: 3v3 → 6v6 → 9v9 → 12v12
Strategy: Greedy vs Reactive
Games: 200 per configuration
Purpose: Understand complexity impact
```

### 4. Fatigue Impact
```text
Configuration: Default game, fatigue enabled/disabled
Strategies: FatigueAware vs Reactive
Games: 1000
Purpose: Measure fatigue strategy effectiveness
```

### 5. Approximation Quality
```text
Configuration: Medium game (6v6, 3 ropes, 20 turns)
Strategies: Exact DP vs ApproxDP (various horizons)
Games: 200
Purpose: Validate approximation methods
```

## Reproducibility

### Essential Information
To reproduce an experiment, record:
```text
- Git commit hash
- Rust version
- Configuration parameters
- Random seed
- Hardware specifications
- Operating system
```

### Seed Management
Always use a fixed random seed for reproducibility:
```text
seed = 42  // Fixed seed
seed = 42 + game_index  // Variable but deterministic
```

## Custom Experiments

### Creating Custom Strategies
```rust
pub struct CustomStrategy {
    parameter: f64,
}

impl Strategy for CustomStrategy {
    fn decide(&mut self, state: &GameState, team: Team) -> TeamAction {
        // Custom logic here
        TeamAction::empty()
    }
    
    fn name(&self) -> &str {
        "Custom"
    }
}
```

### Custom Game Configuration
```rust
pub fn create_custom_game() -> GameState {
    let rules = GameRules {
        duration: 120,
        dt: 0.5,
        position_range: 20,
        // ... custom rules
    };
    
    let ropes = vec![
        Rope::new(0, 0, -20, 20, 5.0, 10),
        // ... custom ropes
    ];
    
    GameState::new(players, ropes, rules, seed)
}
```

## Troubleshooting

### Common Issues

#### Experiment Runner Hangs
```text
Cause: State space too large for DP
Solution: Use approximate methods or smaller games
```

#### Inconsistent Results
```text
Cause: Random seed not fixed
Solution: Ensure random_seed is set in config
```

#### Memory Errors
```text
Cause: Too many threads or large states
Solution: Reduce thread count or use smaller games
```

#### Strategy Always Loses
```text
Cause: Strategy may be fundamentally flawed
Solution: Validate against oracle on small games
```

## Advanced Analysis

### Heatmap Generation
```text
Analyze win rate vs:
- Game duration
- Number of ropes
- Player count
- Rope weights
```

### Correlation Analysis
```text
Correlate:
- Final score with final energy
- Switches with win rate
- Rope control with score
```

### Decision Tree Analysis
```text
Extract decision sequences from strategies
Identify critical decision points
Analyze what distinguishes winners from losers
```

## Batch Experiment Script

### Running Multiple Experiments Overnight
```rust
let experiments = vec![
    ("DP vs Greedy", dp_vs_greedy_config),
    ("DP vs Reactive", dp_vs_reactive_config),
    ("Fatigue vs Greedy", fatigue_vs_greedy_config),
];

for (name, config) in experiments {
    let runner = ExperimentRunner::new(config);
    let results = runner.run();
    export_results(&results, &format!("{}.csv", name))?;
}
```

This framework enables systematic experimentation and strategy comparison for the tug-of-war simulator.