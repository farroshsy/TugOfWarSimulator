# Tug-of-War Dynamic Programming Simulator

> Real-world physical game → mathematical model → dynamic programming → simulation → experiments

A realistic Rust-based simulation that serves as an experimental laboratory for studying how algorithmic models encounter real-world complexity. Starting from a simple finite-state game, the simulator progressively introduces fatigue, heterogeneous players, switching costs, communication delays, incomplete information, and adversarial agents.

---

## The Big Picture

```text
                    REAL-WORLD GAME
                          │
                          ▼
              ┌──────────────────────┐
              │   Tug-of-War Model   │
              │ ropes • players •    │
              │ fatigue • time       │
              └──────────┬───────────┘
                         │
                         ▼
                 Mathematical Model
                         │
             ┌───────────┼───────────┐
             ▼           ▼           ▼
           STATE       ACTION      REWARD
             │           │           │
             └───────────┼───────────┘
                         ▼
                  Dynamic Programming
                         │
             ┌───────────┼───────────┐
             ▼           ▼           ▼
           Exact       Approx.      Other
            DP           DP        Strategies
             │           │           │
             └───────────┼───────────┘
                         ▼
                    SIMULATION
                         │
                         ▼
                  EXPERIMENTS
                         │
                         ▼
                 EMPIRICAL RESULTS
```

---

## Why This Project?

Dynamic programming is usually taught using deliberately clean problems: grids, knapsack, scheduling, paths, and similar abstractions.

Real-world problems are rarely that clean.

This project uses a physical tug-of-war game as a controlled laboratory for studying what happens when algorithmic models encounter real-world complexity.

Starting from a simple finite-state game, the simulator progressively introduces fatigue, heterogeneous players, switching costs, communication delays, incomplete information, stochastic behavior, and adversarial agents.

Each additional mechanic forces a question:

**What information must now become part of the state?**

This makes the project less about implementing one DP algorithm and more about learning how to transform a messy real-world system into a formally solvable optimization problem.

---

## The Game

### Visual Representation

```
TEAM A                                      TEAM B

 A A A                                       B B B
   \                                           /
    \                                         /
─────┬───────────────┬───────────────────────┬─────
     │      R1       │        R2             │ R3
     │    [⚙ ⚙]      │      [⚙ ⚙ ⚙]         │
─────┴───────────────┴───────────────────────┴─────
           ←──────────────→
             rope position
```

### Rope Properties

```
ROPE 1       ROPE 2       ROPE 3
Weight: 1    Weight: 2    Weight: 3
Value:  1    Value:  2    Value:  3
Position:    Position:    Position:
-3.2         +1.7         -0.4
```

### Core Mechanics

- **Teams**: Two teams (A and B) compete over multiple weighted ropes
- **Players**: Each team has players with strength, stamina, and fatigue characteristics
- **Ropes**: Each rope has a position, weight, and point value
- **Objective**: Pull ropes into your territory to score points
- **Time**: Game lasts for a finite number of turns
- **Fatigue**: Players tire while pulling and recover while resting

---

## From Reality to DP

The central challenge: **What information from the past is actually necessary to make the next decision?**

### State Definition

**Basic Exact DP**
```
S = (t, x₁, x₂, ..., x_R)
```
where `t` = time, `x_i` = rope position

**Fatigue-Aware DP**
```
S = (t, x₁, ..., x_R, E_A, E_B)
```
where `E_A`, `E_B` = team energy (compressed)

Each additional state variable changes the computational complexity of the exact solution.

### Action Space

```
action = (a₁, a₂, ..., a_R)
```
where `a_i` = number of players on rope i, `Σ a_i = P`

Number of allocations: `C(P + R - 1, R - 1)`

### Transition Function

```
T: S × A_A × A_B → S'
```

1. Apply player movements
2. Calculate pulling forces
3. Update rope positions
4. Update stamina (fatigue/recovery)
5. Advance time

### Bellman Recurrence

```
V_t(S) = max_{a_A ∈ A_A(S)} min_{a_B ∈ A_B(S)} V_{t+1}(T(S, a_A, a_B))
```

Team A maximizes, Team B minimizes (finite-horizon minimax).

---

## Model Evolution

```text
                    MODEL EVOLUTION

       ┌─────────────────────────────┐
       │ Simple Tug-of-War           │
       │ position + time             │
       └──────────────┬──────────────┘
                      │
                      ▼
       ┌─────────────────────────────┐
       │ Multiple Ropes              │
       │ + rope positions            │
       └──────────────┬──────────────┘
                      │
                      ▼
       ┌─────────────────────────────┐
       │ Multiple Players            │
       │ + allocations               │
       └──────────────┬──────────────┘
                      │
                      ▼
       ┌─────────────────────────────┐
       │ Fatigue                     │
       │ + energy                    │
       └──────────────┬──────────────┘
                      │
                      ▼
       ┌─────────────────────────────┐
       │ Heterogeneous Players       │
       │ + individual attributes     │
       └──────────────┬──────────────┘
                      │
                      ▼
       ┌─────────────────────────────┐
       │ Switching / Coordination    │
       │ + action constraints        │
       └──────────────┬──────────────┘
                      │
                      ▼
       ┌─────────────────────────────┐
       │ Commander / Delayed Info    │
       │ + communication state       │
       └──────────────┬──────────────┘
                      │
                      ▼
       ┌─────────────────────────────┐
       │ Spies / Hidden Information  │
       │ + hidden state              │
       └──────────────┬──────────────┘
                      │
                      ▼
                ┌────────────┐
                │ DP becomes │
                │ insufficient│
                │ / changes  │
                └────────────┘
```

---

## State Explosion

The central problem is not writing the recurrence. It is controlling the size of the state.

```text
                 STATE SPACE

Basic
████

+ ropes
████████

+ players
██████████████████

+ fatigue
████████████████████████████

+ heterogeneous players
████████████████████████████████████████

+ communication
████████████████████████████████████████████████

+ hidden information
████████████████████████████████████████████████████████████
```

### Complexity Summary

| Configuration | States | Actions | Time Complexity | Exact DP |
|--------------|-------|---------|----------------|----------|
| 3v3, 2 ropes, 10 turns | 1,210 | 4 | ~2,000 | ✓ |
| 6v6, 3 ropes, 20 turns | 185,220 | 28 | ~145M | ⚠️ |
| 12v12, 3 ropes, 60 turns | 555,660 | 91 | ~4.6B | ❌ |
| + fatigue (20 buckets) | 222M | 91 | ~18B | ❌ |

---

## Architecture

### Engine/Solver Separation

```text
                         ┌───────────────┐
                         │ Configuration │
                         └───────┬───────┘
                                 │
                                 ▼
┌───────────┐              ┌──────────────┐
│ Strategy A│─────────────▶│              │
└───────────┘              │ Game Engine  │
                           │              │
┌───────────┐              │              │
│ Strategy B│─────────────▶│              │
└───────────┘              └──────┬───────┘
                                  │
                                  ▼
                           ┌─────────────┐
                           │ Game State  │
                           └──────┬──────┘
                                  │
                    ┌─────────────┼─────────────┐
                    ▼             ▼             ▼
                 Physics      Fatigue       Scoring
                    │             │             │
                    └─────────────┼─────────────┘
                                  ▼
                             Experiment
                               Runner
                                  │
                    ┌─────────────┼─────────────┐
                    ▼             ▼             ▼
                 Win Rate      Score        Energy
```

### Strategy Layer

```text
                    STRATEGY LAYER
                         │
        ┌────────────────┼────────────────┐
        ▼                ▼                ▼
      Greedy          Reactive            DP
                                           │
                                  ┌────────┴────────┐
                                  ▼                 ▼
                              Exact DP         Approx DP
```

---

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
  - `FatigueDpStrategy`: Fatigue-aware DP with compressed energy state
  - `ApproxDPStrategy`: Limited horizon beam search for large games

### Dynamic Programming
- **Exact Minimax DP**: Optimal finite-horizon minimax with memoization
- **Allocation Generation**: Efficient combinatorial algorithm for player distributions
- **Feasibility Estimation**: State space analysis to predict computational requirements
- **Policy Extraction**: Store optimal actions for each state
- **Fatigue-Aware DP**: Extended state with compressed energy buckets
- **Approximate Methods**: Limited horizon beam search for large state spaces

### Advanced Mechanics
- **Heterogeneous Players**: Strong, Normal, Weak player classes with different attributes
- **Switching Constraints**: Configurable limits and costs for rope switching
- **Commander System**: Communication delay and command cooldown mechanics
- **Information Models**: Perfect, local, and delayed information options
- **Spy System**: Hidden adversarial agents with configurable behavior

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

---

## Research Questions

This project explores questions such as:

1. How does the definition of the state change as realism increases?
2. Which physical variables are necessary for an optimal policy?
3. When does exact dynamic programming become computationally infeasible?
4. How much performance is lost through state compression?
5. When does a problem stop being naturally expressible as deterministic DP?
6. How do greedy, reactive, fatigue-aware, exact DP, and approximate strategies behave under different game configurations?
7. How does player heterogeneity affect optimal allocation?
8. How valuable is information about the opponent?
9. How do communication delays affect coordination?
10. Can a strategy discovered in simulation produce useful insight about the corresponding real-world game?

---

## Documentation

This project includes comprehensive mathematical and methodological documentation:

- **[docs/real-world-to-dp.md](docs/real-world-to-dp.md)**: Systematic methodology for converting real-world problems into DP formulations
- **[docs/mathematical-model.md](docs/mathematical-model.md)**: Complete mathematical formulation of the tug-of-war model
- **[docs/dp-formulation.md](docs/dp-formulation.md)**: State, action, transition, recurrence, base cases, policy, complexity
- **[docs/pseudocode.md](docs/pseudocode.md)**: Language-independent pseudocode for all algorithms
- **[docs/model-evolution.md](docs/model-evolution.md)**: How the model changes as features are added progressively
- **[docs/complexity.md](docs/complexity.md)**: Detailed state/action-space analysis and scalability limits
- **[docs/validation.md](docs/validation.md)**: Systematic validation methodology for models, algorithms, and implementations
- **[docs/experiments.md](docs/experiments.md)**: Guide to running experiments and comparing strategies
- **[docs/problem-modeling-template.md](docs/problem-modeling-template.md)**: Reusable worksheet for new problems

---

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
│   ├── factory.rs         # Game creation helpers
│   ├── player_class.rs     # Heterogeneous player classes
│   ├── switching.rs        # Switching constraints
│   ├── commander.rs        # Commander mechanics
│   ├── information.rs      # Information models
│   └── spy.rs              # Spy mechanics
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
│   ├── dp.rs               # Minimax DP solver
│   ├── fatigue_dp.rs       # Fatigue-aware DP
│   └── approximate_dp.rs   # Limited horizon DP
├── experiments/            # Batch testing
│   ├── mod.rs
│   ├── runner.rs           # Experiment execution
│   ├── metrics.rs          # Statistics, comparison tables
│   └── export.rs           # CSV export
└── ui/                     # GUI components
    ├── mod.rs
    └── app.rs              # Main application
```

---

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

---

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
    strength_variation: 0.2,
};

let runner = ExperimentRunner::new(config);
let results = runner.run();
println!("{}", results);
```

---

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

---

## Experimental Results

> **Status:** Example output shown below. These numbers are illustrative and should not be interpreted as scientifically validated results unless accompanied by the corresponding experiment configuration, seed, model version, and reproducibility information.

Example results from strategy comparison (small game, 100 games each, seed 42, `strength_variation: 0.2`):

```
Strategy Comparison
====================
Strategy                 Wins   Losses    Draws Win Rate  Avg Score
--------                 ----   ------    ----- --------  ---------
Random vs Greedy            1       99        0     1.0      -0.96
Greedy vs Reactive         24       26       50    24.0       0.04
Reactive vs FatigueAware       40       10       50    40.0       0.42
FatigueAware vs Random       97        3        0    97.0       0.94
```

---

## Performance

### DP Solver Performance
- **Small game (3v3, 2 ropes, 10 turns)**: ~7ms, 91 states cached
- **Medium game (4v4, 3 ropes, 20 turns)**: ~500ms, ~1,000 states cached
- **Large game (12v12, 3 ropes, 60 turns)**: Infeasible for exact DP

### Experiment Runner
- **100 games**: ~2 seconds (multi-threaded)
- **1,000 games**: ~15 seconds
- **10,000 games**: ~2 minutes

---

## Project Status

### Completed Phases
- ✅ Phase 1: Core simulation with realistic physics
- ✅ Phase 2: Strategy framework and baseline algorithms  
- ✅ Phase 3: Interactive GUI with arena visualization
- ✅ Phase 4: Exact DP solver with feasibility analysis
- ✅ Phase 5: Experiment framework and batch testing
- ✅ Phase 6: Fatigue system and fatigue-aware DP
- ✅ Phase 7: Advanced mechanics (heterogeneous players, switching, commander)
- ✅ Phase 8: Advanced game theory (partial information, spies, approximate DP)

### Key Achievements
1. **Clean Architecture**: Perfect separation between game engine and strategy solvers
2. **Realistic Simulation**: Configurable complexity from simple to complex
3. **Working DP**: Exact minimax DP works for small state spaces with feasibility analysis
4. **Experimental Platform**: Strategy comparison and batch testing capabilities
5. **Interactive UI**: Real-time visualization and control
6. **Comprehensive Documentation**: Mathematical methodology and validation procedures

---

## Future Research Directions

- Continuous-state control
- Stochastic dynamic programming
- POMDP formulation
- Reinforcement learning
- Monte Carlo Tree Search
- Multi-agent reinforcement learning
- Real-world parameter calibration
- Human strategy experiments

---

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

---

## Validation & Reproducibility

This project includes systematic validation procedures documented in [docs/validation.md](docs/validation.md). All experiments should record:
- Git commit hash
- Rust version
- Configuration parameters
- Random seed
- Hardware specifications
- Operating system

---

## Contributing

This is a research/educational project. Contributions are welcome, especially:
- Additional strategy implementations
- Approximate DP methods
- Enhanced UI features
- Performance optimizations
- Documentation improvements

---

## License

This project is provided as-is for educational and research purposes.

---

## Acknowledgments

This project demonstrates the application of dynamic programming and game theory to a real-world physical game, showing how algorithmic decision-making can be studied through realistic simulation.