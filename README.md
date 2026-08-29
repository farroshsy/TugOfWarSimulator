# Tug-of-War Dynamic Programming Simulator

A realistic Rust-based simulation of the tug-of-war game that serves as an experimental platform for applying dynamic programming, adversarial search, resource allocation, and game-theoretic methods to a real-world physical game.

## Overview

This project demonstrates how a real-world physical game can be decomposed into:
- **State**: Rope positions, time, player energy
- **Actions**: Player allocation to ropes
- **Transitions**: Physics-based rope movement
- **Constraints**: Fatigue, switching limits, coordination
- **Optimization Objectives**: Score maximization

The simulator maintains a clean separation between the **game engine** and **strategy solvers**, allowing different algorithms (Greedy, Reactive, DP, etc.) to compete on the same realistic simulation.

## Features

### Core Simulation
- **Realistic Physics**: Weight-based rope movement with configurable thresholds
- **Player Model**: Individual strength, stamina, fatigue rates, recovery rates
- **Game Mechanics**: Time advancement, scoring, rope ownership
- **Configurable Rules**: Game duration, position ranges, switching constraints

### Strategy Framework
- **Strategy Trait**: Common interface for all decision-making approaches
- **Implemented Strategies**:
  - `RandomStrategy`: Random player allocation
  - `GreedyStrategy`: Always allocate to highest-value rope
  - `ReactiveStrategy`: Consider rope position and opponent allocation
  - `FatigueAwareStrategy`: Conserve energy early, commit late
  - `DPStrategy`: Exact minimax dynamic programming (small games)

### Dynamic Programming
- **Exact Minimax DP**: Optimal finite-horizon minimax with memoization
- **Allocation Generation**: Efficient combinatorial algorithm for player distributions
- **Feasibility Estimation**: State space analysis to predict computational requirements
- **Policy Extraction**: Store optimal actions for each state
- **State Space**: `DP(time, rope_positions)` for exact solution

### Experiment Framework
- **Batch Simulation**: Run thousands of games with multi-threading
- **Statistics Collection**: Win rates, scores, energy usage, rope control
- **Strategy Comparison**: Tabular comparison of algorithm performance
- **CSV Export**: Export results for further analysis

### Interactive UI
- **egui/eframe Interface**: Modern Rust GUI framework
- **Arena Visualization**: Real-time rope position display
- **Player Panels**: Team composition and energy monitoring
- **Game Controls**: Start, pause, step, reset with speed control
- **Strategy Selection**: Dynamic strategy switching between games

## Installation

### Prerequisites
- Rust 1.70 or later
- Cargo

### Build
```bash
cargo build --release
```

### Run
```bash
# Run the GUI application
cargo run --release

# Run simulation tests
cargo run --bin test_simulation

# Run DP solver tests
cargo run --bin test_dp

# Run experiment tests
cargo run --bin test_experiments
```

## Project Structure

```
src/
├── main.rs                 # GUI application entry point
├── lib.rs                  # Library exports
├── game/                   # Core simulation
│   ├── mod.rs
│   ├── state.rs            # GameState, time management
│   ├── player.rs           # Player attributes, fatigue
│   ├── rope.rs             # Rope properties, movement
│   ├── rules.rs            # Game configuration
│   ├── physics.rs          # Force calculation, movement
│   ├── simulation.rs       # Game step, scoring
│   └── factory.rs         # Game creation helpers
├── strategies/             # Decision-making algorithms
│   ├── mod.rs
│   ├── strategy.rs         # Strategy trait, actions
│   ├── random.rs           # Random strategy
│   ├── greedy.rs           # Greedy strategy
│   ├── reactive.rs         # Reactive strategy
│   └── fatigue.rs          # Fatigue-aware strategy
├── algorithms/             # DP and optimization
│   ├── mod.rs
│   ├── allocation.rs       # Allocation generation
│   └── dp.rs               # Minimax DP solver
├── experiments/            # Batch testing
│   ├── mod.rs
│   ├── runner.rs           # Experiment execution
│   ├── metrics.rs          # Statistics, comparison tables
│   └── export.rs           # CSV export
└── ui/                     # GUI components
    ├── mod.rs
    └── app.rs              # Main application
```

## Usage Examples

### Basic Simulation
```rust
use tug_of_war::game::{create_default_game};
use tug_of_war::strategies::{GreedyStrategy, ReactiveStrategy, Strategy};
use tug_of_war::game::simulation::{step, calculate_score};

let mut state = create_default_game();
let mut strategy_a = GreedyStrategy::new();
let mut strategy_b = ReactiveStrategy::new();

while !state.finished() {
    let action_a = strategy_a.decide(&state, Team::A);
    let action_b = strategy_b.decide(&state, Team::B);
    step(&mut state, &action_a, &action_b);
}

let score = calculate_score(&state);
println!("Final score: A={}, B={}", score.team_a, score.team_b);
```

### Dynamic Programming
```rust
use tug_of_war::algorithms::{DpSolver, DpFeasibilityEstimator};

// Check feasibility
let estimator = DpFeasibilityEstimator::new(3, 2, 10, 5);
println!("{}", estimator.report());

// Solve exactly
let mut solver = DpSolver::new(3, 2);
let value = solver.solve(&state);
println!("Optimal value: {}", value);

// Get optimal action
if let Some(allocation) = solver.get_optimal_action(&state) {
    println!("Optimal allocation: {:?}", allocation.counts);
}
```

### Batch Experiments
```rust
use tug_of_war::experiments::{ExperimentRunner, ExperimentConfig, StrategyType, GameSize};

let config = ExperimentConfig {
    number_of_games: 1000,
    random_seed: 42,
    team_a_strategy: StrategyType::Greedy,
    team_b_strategy: StrategyType::Reactive,
    game_size: GameSize::Small,
};

let runner = ExperimentRunner::new(config);
let results = runner.run();
println!("{}", results);
```

## Mathematical Foundation

### DP State Definition
For the basic exact DP, the state is:
```
DP(t, x₁, x₂, ..., x_R)
```
where:
- `t` = current time step
- `x_i` = position of rope i

### Bellman Recurrence
```
V(t, state) = max_A min_B V(t+1, next_state)
```
Team A maximizes, Team B minimizes (minimax).

### Base Case
```
V(T, state) = Score_A - Score_B
```

### Complexity Analysis
For P people per team and R ropes:
- **Allocations per team**: `C(P + R - 1, R - 1)`
- **Joint actions per state**: `allocations²`
- **States**: `T × (2M + 1)^R` (where M = position range)
- **Time complexity**: `O(T × (2M + 1)^R × C(P + R - 1, R - 1)²)`

### Example Complexity
For 12 people, 3 ropes, 21 positions, 60 time steps:
- Allocations per team: 91
- Joint actions: 8,281
- States: 60 × 21³ = 555,660
- Estimated transitions: ~4.6 billion

This explains why exact DP only works for small configurations.

## Model Assumptions

The current implementation makes these simplifying assumptions for exact DP:

✅ **Enabled**:
- Discrete time steps
- Discrete rope positions
- Perfect information
- Deterministic physics
- Simultaneous team decisions
- Finite horizon

❌ **Disabled** (for future expansion):
- Hidden information
- Random spy behavior
- Individual stochastic fatigue
- Continuous time
- Continuous rope positions

## Experimental Results

Example results from strategy comparison (100 games each):

```
Strategy Comparison
====================
Strategy                 Wins   Losses    Draws Win Rate  Avg Score
--------                 ----   ------    ----- --------  ---------
Random vs Greedy          100        0        0   100.0       1.00
Greedy vs Reactive          0        0      100     0.0       0.00
Reactive vs FatigueAware        0        0      100     0.0       0.00
FatigueAware vs Random        0      100        0     0.0      -1.00
```

## Configuration

Default game configuration:
- **Teams**: 2 (A and B)
- **Players per team**: 12
- **Ropes**: 3 (weights: 1, 2, 3; values: 1, 2, 3)
- **Duration**: 60 time steps
- **Position range**: ±10 units
- **Switching limit**: 4 players per turn
- **Fatigue**: Enabled

Small game (for DP testing):
- **Players per team**: 3
- **Ropes**: 2
- **Duration**: 10 time steps
- **Position range**: ±5 units

## Future Extensions

### Phase 6: Advanced DP
- Fatigue-aware DP with compressed energy state
- State discretization for continuous variables

### Phase 7: Advanced Mechanics
- Heterogeneous player classes (Strong, Normal, Weak)
- Switching costs and coordination constraints
- Commander mechanics with communication delay

### Phase 8: Advanced Game Theory
- Partial information models
- Spy/traitor mechanics
- Approximate DP methods (MCTS, beam search)
- Stochastic game formulations

## Performance

### DP Solver Performance
- **Small game (3v3, 2 ropes, 10 turns)**: ~7ms, 91 states cached
- **Medium game (4v4, 3 ropes, 20 turns)**: ~500ms, ~1,000 states cached
- **Large game (12v12, 3 ropes, 60 turns)**: Infeasible for exact DP

### Experiment Runner
- **100 games**: ~2 seconds (multi-threaded)
- **1,000 games**: ~15 seconds
- **10,000 games**: ~2 minutes

## Contributing

This is a research/educational project. Contributions are welcome, especially:
- Additional strategy implementations
- Approximate DP methods
- Enhanced UI features
- Performance optimizations
- Documentation improvements

## License

This project is provided as-is for educational and research purposes.

## Acknowledgments

This project demonstrates the application of dynamic programming and game theory to a real-world physical game, showing how algorithmic decision-making can be studied through realistic simulation.