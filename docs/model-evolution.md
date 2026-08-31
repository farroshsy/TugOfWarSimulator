# Model Evolution Documentation

## Overview

This document tracks how the tug-of-war mathematical model evolves as we progressively add realistic features. Each addition changes the state space, action space, constraints, transition function, and computational complexity.

## Level 0: Basic Model

### Configuration
```text
Players per team: 1
Ropes: 1
Duration: 5 turns
Position range: ±2
```

### State
```text
S_t = (t, x_1)
```

### Actions
```text
A = {(1)}  // Only one action: pull the single rope
```

### Transition
```text
x_1(t+1) = x_1(t) + sign(F_1(t))
```

### Complexity
```text
States: 5 × 5 = 25
Actions: 1
Transitions: 25
```

### Mathematical Type
Single-agent, deterministic, finite-horizon optimization

---

## Level 1: Multiple Ropes

### Configuration
```text
Players per team: 1
Ropes: 3
Duration: 10 turns
Position range: ±5
```

### State
```text
S_t = (t, x_1, x_2, x_3)
```

### Actions
```text
A = {(1,0,0), (0,1,0), (0,0,1)}  // Choose which rope to pull
```

### Number of Actions
```text
|A| = C(1 + 3 - 1, 3 - 1) = C(3,2) = 3
```

### Transition
```text
x_i(t+1) = x_i(t) + sign(F_i(t)) for chosen rope i
```

### Complexity
```text
States: 10 × 11³ = 13,310
Actions: 3
Joint actions: 9
Transitions: 119,790
```

### Mathematical Type
Single-agent, deterministic, finite-horizon optimization

---

## Level 2: Multiple Players

### Configuration
```text
Players per team: 3
Ropes: 2
Duration: 10 turns
Position range: ±5
```

### State
```text
S_t = (t, x_1, x_2)
```

### Actions
```text
A = {(a_1, a_2) : a_1 + a_2 = 3, a_i ≥ 0}
```

### Number of Actions
```text
|A| = C(3 + 2 - 1, 2 - 1) = C(4,1) = 4
```

Possible allocations: (3,0), (2,1), (1,2), (0,3)

### Transition
```text
F_i(t) = Σ_{j∈A_i} σ_j - Σ_{j∈B_i} σ_j
x_i(t+1) = x_i(t) + sign(F_i(t))
```

### Complexity
```text
States: 10 × 11² = 1,210
Actions: 4
Joint actions: 16
Transitions: 19,360
```

### Mathematical Type
Single-agent, deterministic, finite-horizon optimization

---

## Level 3: Two Teams (Adversarial)

### Configuration
```text
Players per team: 3
Ropes: 2
Duration: 10 turns
Position range: ±5
```

### State
```text
S_t = (t, x_1, x_2)
```

### Actions
```text
A_A = {(a_1, a_2) : a_1 + a_2 = 3}
A_B = {(b_1, b_2) : b_1 + b_2 = 3}
```

### Bellman Recurrence
```text
V_t(S) = max_{a_A ∈ A_A} min_{a_B ∈ A_B} V_{t+1}(T(S, a_A, a_B))
```

### Complexity
```text
States: 10 × 11² = 1,210
Actions per team: 4
Joint actions: 16
Transitions: 19,360
```

### Mathematical Type
Two-player zero-sum, deterministic, finite-horizon minimax DP

---

## Level 4: Fatigue

### Configuration
```text
Players per team: 3
Ropes: 2
Duration: 10 turns
Position range: ±5
Fatigue: Enabled
```

### State
```text
S_t = (t, x_1, x_2, E_A, E_B)
```

### Additional Variables
```text
E_A = Σ_j s_j^A  // Team A total energy
E_B = Σ_j s_j^B  // Team B total energy
```

### Transition Extension
```text
F_i(t) = Σ_{j∈A_i} σ_j × (s_j/S_max) - Σ_{j∈B_i} σ_j × (s_j/S_max)
s_j(t+1) = s_j(t) - f_j if pulling
s_j(t+1) = s_j(t) + r_j if resting
```

### Complexity
```text
States: 10 × 11² × 20² = 48,400 (assuming 20 energy buckets)
Actions: 4
Joint actions: 16
Transitions: 774,400
```

### Mathematical Type
Two-player zero-sum, deterministic, finite-horizon minimax DP with resource constraints

---

## Level 5: Rope Weight

### Configuration
```text
Players per team: 3
Ropes: 2
Duration: 10 turns
Position range: ±5
Rope weights: w_1=1, w_2=2
```

### State
```text
S_t = (t, x_1, x_2, E_A, E_B)
```

### Transition Extension
```text
Δx_i(t) = 0 if |F_i(t)| ≤ w_i
Δx_i(t) = sign(F_i(t)) if |F_i(t)| > w_i
```

### Mathematical Impact
Heavier ropes require more force to move, creating strategic tradeoffs.

### Complexity
```text
States: 48,400 (same as Level 4)
Actions: 4
Joint actions: 16
Transitions: 774,400
```

### Mathematical Type
Two-player zero-sum, deterministic, finite-horizon minimax DP with weighted objectives

---

## Level 6: Heterogeneous Players

### Configuration
```text
Players per team: 3 (1 Strong, 1 Normal, 1 Weak)
Ropes: 2
Duration: 10 turns
Strong: σ=10, S_max=100, f=1.8
Normal: σ=7, S_max=100, f=1.2
Weak: σ=5, S_max=120, f=0.8
```

### State Options

#### Option A: Individual Tracking
```text
S_t = (t, x_1, x_2, s_1^A, s_2^A, s_3^A, s_1^B, s_2^B, s_3^B)
```

#### Option B: Class Aggregation
```text
S_t = (t, x_1, x_2, E_strong^A, E_normal^A, E_weak^A, E_strong^B, E_normal^B, E_weak^B)
```

#### Option C: Team Energy (Current)
```text
S_t = (t, x_1, x_2, E_A, E_B)
```

### Complexity Comparison
```text
Individual: 10 × 11² × 101⁶ ≈ 1.2×10¹² states (infeasible)
Class-based: 10 × 11² × 20⁶ ≈ 2.4×10⁸ states (difficult)
Team energy: 10 × 11² × 20² = 48,400 states (feasible)
```

### Mathematical Type
Two-player zero-sum, deterministic, finite-horizon minimax DP with heterogeneous agents

---

## Level 7: Switching Constraints

### Configuration
```text
Players per team: 3
Ropes: 2
Duration: 10 turns
Max switches per turn: K=2
Switching cost: C=5 stamina
```

### State Extension
```text
S_t = (t, x_1, x_2, E_A, E_B, assignments_A, assignments_B)
```

### Action Constraints
```text
A_A(s) = {allocations satisfying:
    Σ a_i = P AND
    |{j: a_j ≠ current_assignment_j}| ≤ K}
```

### Complexity Impact
```text
States: 10 × 11² × 20² × C(3,2)² ≈ 1.3×10⁶
Actions: 4 → 2 (switching reduces legal actions)
Transitions: varies
```

### Mathematical Type
Two-player zero-sum, deterministic, finite-horizon minimax DP with action constraints

---

## Level 8: Commander Delay

### Configuration
```text
Players per team: 3
Ropes: 2
Duration: 10 turns
Command cooldown: 3 turns
Command delay: 1 turn
```

### State Extension
```text
S_t = (t, x_1, x_2, E_A, E_B, cooldown_A, cooldown_B, pending_commands_A, pending_commands_B)
```

### Transition Extension
```text
// Commands issued at t execute at t+d
Execute pending commands before calculating forces
```

### Complexity Impact
```text
States: significantly increased due to command queue state
Actions: unchanged
Transitions: more complex due to delayed effects
```

### Mathematical Type
Two-player zero-sum, deterministic, finite-horizon minimax DP with delayed actions

---

## Level 9: Partial Information

### Configuration
```text
Players per team: 3
Ropes: 2
Duration: 10 turns
Information model: Local
```

### State (Game Reality)
```text
S_t = (t, x_1, x_2, E_A, E_B, ...)
```

### Observation (What Strategy Sees)
```text
O_t = {x_j(t), F_j(t), opponent_allocations_j(t)}
```
where j = current rope of observing player

### Mathematical Type
Two-player zero-sum, deterministic, finite-horizon **partially observable** minimax DP (now requires belief states)

---

## Level 10: Spies

### Configuration
```text
Players per team: 3
Ropes: 2
Duration: 10 turns
Spy probability: p=0.2
Spy behavior: Stochastic
```

### State (Reality)
```text
S_t = (t, x_1, x_2, E_A, E_B, spy_states)
```

### State (Strategy Belief)
```text
B_t = probability distribution over which players are spies
```

### Mathematical Type
Two-player zero-sum, **stochastic** minimax DP with **hidden state** (now requires Bayesian game theory)

---

## Level 11: Full Realistic Game

### Configuration
```text
Players per team: 12
Ropes: 3
Duration: 60 turns
Position range: ±10
Fatigue: Enabled
Heterogeneous players: Enabled
Switching constraints: Enabled
Commander delay: Enabled
Partial information: Enabled
Spies: Enabled
```

### State Space (Full)
```text
S_t = (t, x_1, x_2, x_3, s_1, ..., s_24, assignments, commands, spy_states, ...)
```

### Estimated State Count
```text
Continuous variables: 24 stamina values × 101 possibilities each
Discrete variables: 21³ × various other dimensions
Total: > 10²⁴ states (completely infeasible)
```

### Mathematical Type
**Not DP** - This is a partially observable stochastic game with enormous state space

---

## Complexity Summary Table

| Level | State Variables | States | Actions | Type | Exact DP |
|-------|----------------|-------|---------|------|----------|
| 0 | t, x_1 | 25 | 1 | Optimization | ✓ |
| 1 | t, x_1, x_2, x_3 | 13,310 | 3 | Optimization | ✓ |
| 2 | t, x_1, x_2 | 1,210 | 4 | Optimization | ✓ |
| 3 | t, x_1, x_2 | 1,210 | 4 | Minimax | ✓ |
| 4 | t, x_1, x_2, E_A, E_B | 48,400 | 4 | Minimax + Resources | ✓ |
| 5 | t, x_1, x_2, E_A, E_B (weighted) | 48,400 | 4 | Minimax + Weighted | ✓ |
| 6 | t, x_1, x_2, E_A, E_B (class) | 2.4×10⁸ | 4 | Minimax + Heterogeneous | ⚠️ |
| 7 | t, x_1, x_2, E_A, E_B, assignments | 1.3×10⁶ | 2 | Minimax + Constraints | ⚠️ |
| 8 | t, x_1, x_2, E_A, E_B, commands | >10⁶ | 4 | Minimax + Delayed | ⚠️ |
| 9 | t, x_1, x_2, E_A, E_B (partial obs) | 48,400 | 4 | POMDP | ❌ |
| 10 | t, x_1, x_2, E_A, E_B, spies | 48,400 | 4 | Stochastic Game | ❌ |
| 11 | full realistic | >10²⁴ | 91 | Complex Game | ❌ |

## Key Insights

### 1. State Explosion is Real
Each realistic feature can multiply the state space by 10-100× or more.

### 2. Not All Features Need Full State
- Fatigue can be compressed to team energy
- Player heterogeneity can be aggregated by class
- Information constraints affect the strategy, not necessarily the state

### 3. Problem Type Changes
- Adding randomness → Stochastic DP/MDP
- Adding hidden information → POMDP
- Adding hidden adversaries → Bayesian/Stochastic games
- Adding continuous variables → Control theory

### 4. Approximation Becomes Necessary
For realistic games, we must use:
- State aggregation
- Limited horizon planning
- Monte Carlo methods
- Heuristic strategies

### 5. Value of Progressive Complexity
Starting simple and adding features gradually allows us to:
- Validate each component independently
- Understand the mathematical impact of each feature
- Identify when exact DP becomes infeasible
- Choose appropriate approximation methods

This evolution demonstrates the fundamental challenge of applying DP to real-world problems: **the gap between mathematical purity and physical reality**.