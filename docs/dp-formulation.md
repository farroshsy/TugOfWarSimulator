# DP Formulation for Tug-of-War

## State Definition

### Canonical State Representation
```rust
#[derive(Clone, Hash, Eq, PartialEq)]
struct DpState {
    time: u8,
    rope_positions: Vec<i8>,
}
```

### State Variables
```text
S_t = (t, x_1, x_2, ..., x_R)
```

where:
- `t ∈ {0, 1, ..., T}` = current timestep
- `x_i ∈ {-M, ..., 0, ..., M}` = position of rope i
- `M` = maximum rope position

### State Sufficiency Test
> **Question**: If I delete the entire history before time t, can I still determine all possible future transitions and their consequences from S_t?

**Answer**: Yes, for the basic model, because:
- Rope positions determine pulling forces
- Pulling forces determine rope movement
- Rope movement determines future positions
- Player abilities are assumed constant

## Action Definition

### Action Space
```text
A(s) = {allocations satisfying Σ a_i = P}
```

### Allocation Representation
```rust
struct Allocation {
    counts: Vec<usize>, // counts[i] = players on rope i
}
```

### Action Constraints
```text
a_1 + a_2 + ... + a_R = P
a_i ≥ 0 for all i
```

### Switching Constraints (when enabled)
```text
|{j : a_j(t) ≠ a_j(t-1)}| ≤ K
```

## Transition Function

### Mathematical Definition
```text
T: S × A_A × A_B → S'
```

### Implementation
```rust
fn transition(state: &GameState, action_a: &Allocation, action_b: &Allocation) -> GameState {
    let mut next_state = state.clone();
    
    // 1. Apply player movements
    apply_actions(&mut next_state, action_a, action_b);
    
    // 2. Calculate forces
    let forces = calculate_forces(&next_state);
    
    // 3. Update rope positions
    update_positions(&mut next_state, forces);
    
    // 4. Update stamina
    update_stamina(&mut next_state);
    
    // 5. Advance time
    next_state.time += 1;
    
    next_state
}
```

### Force Calculation
```text
F_i = Σ_{j∈A_i} σ_j × (s_j/S_max) - Σ_{j∈B_i} σ_j × (s_j/S_max)
```

### Position Update
```text
x_i' = x_i + sign(F_i) × 1(|F_i| > w_i)
```

## Reward Function

### Terminal Reward
```text
R_T(S) = Σ_{i: x_i < 0} v_i - Σ_{i: x_i > 0} v_i
```

### No Intermediate Rewards
```text
R_t(S, a_A, a_B) = 0 for t < T
```

## Bellman Recurrence

### Two-Player Minimax
```text
V_t(S) = max_{a_A ∈ A_A(S)} min_{a_B ∈ A_B(S)} V_{t+1}(T(S, a_A, a_B))
```

### Explanation
- Team A chooses action to maximize future value
- Team B chooses action to minimize Team A's future value
- This is **not** ordinary single-agent DP

### Base Case
```text
V_T(S) = R_T(S)
```

## Policy Extraction

### Optimal Policy
```text
π(S) = argmax_{a_A ∈ A_A(S)} min_{a_B ∈ A_B(S)} V_{t+1}(T(S, a_A, a_B))
```

### Implementation
```rust
fn extract_policy(solver: &DpSolver, state: &GameState) -> Option<Allocation> {
    solver.policy_cache.get(&state_key(state)).cloned()
}
```

## Base Cases

### Terminal State
```text
IF t == T:
    RETURN R_T(S)
```

### Rope Position Bounds
```text
x_i' = clamp(x_i', -M, M)
```

### Stamina Bounds
```text
s_j' = clamp(s_j', 0, S_max)
```

## Memoization

### Cache Structure
```rust
HashMap<StateKey, f64>  // state → value
HashMap<StateKey, Allocation>  // state → optimal action
```

### Memoization Logic
```text
IF state in cache:
    RETURN cache[state]

value = solve(state)
cache[state] = value
RETURN value
```

## Complexity Analysis

### State Count
```text
|S| = T × (2M + 1)^R
```

### Action Count
```text
|A| = C(P + R - 1, R - 1)
```

### Time Complexity
```text
O(T × (2M + 1)^R × C(P + R - 1, R - 1)²)
```

### Space Complexity
```text
O(T × (2M + 1)^R)
```

## Rust Implementation Mapping

| Mathematics | Rust |
|-------------|------|
| State | `GameState` |
| DP State | `StateKey` |
| Action | `Allocation` |
| Transition | `step()` |
| Reward | `calculate_score()` |
| Value function | `HashMap<StateKey, f64>` |
| Policy | `HashMap<StateKey, Allocation>` |
| Memoization | `DpSolver.cache` |

## Fatigue-Aware Extension

### Extended State
```text
S_t = (t, x_1, ..., x_R, E_A, E_B)
```

### Energy Compression
```text
E_A_compressed = floor(E_A / E_max × buckets)
E_B_compressed = floor(E_B / E_max × buckets)
```

### Complexity Impact
```text
|S| = T × (2M + 1)^R × buckets²
```

For buckets = 20: 400x increase in state space

## Approximate Methods

### Limited Horizon
```text
V_t(S) = max_{a_A} min_{a_B} V_{t+H}(T(S, a_A, a_B))
```
where H << T

### Beam Search
```text
Keep only top K states at each level
```

### Value Approximation
```text
V_t(S) ≈ neural_network(S)
```

## State Space Analysis

### Feasibility Estimator
```text
IF estimated_transitions > threshold:
    EXACT DP INFEASIBLE
    USE APPROXIMATE METHODS
```

### Estimation Formula
```text
transitions ≈ T × (2M + 1)^R × C(P + R - 1, R - 1)²
```

## Key Implementation Details

### State Hashing
```rust
#[derive(Hash, Eq, PartialEq)]
struct StateKey {
    time: usize,
    positions: Vec<i32>,
}
```

### Canonical Representation
Use consistent ordering to ensure equivalent states hash identically.

### Memory Management
```rust
solver.clear_cache()  // Reuse solver for different parameters
```

## Validation Strategy

### Small Hand-Computed Example
```text
P = 2, R = 2, T = 3, M = 1
```

Enumerate all states and verify:
1. Manual calculation matches DP result
2. Recursion matches memoized result
3. Policy matches value function

### Oracle Experiments
```text
Exact DP (small game) vs Approximate DP (same game)
```

Measure:
- Value gap
- Policy disagreement rate
- Win rate difference

## Mathematical Conventions

### Standard Notation
```text
t       = timestep
T       = terminal timestep
s       = state
a       = action
A(s)    = legal action set
T(s,a)  = transition function
R(s,a)  = immediate reward
V_t(s)  = value function
π(s)    = policy
```

### Team Notation
```text
a_A     = Team A action
a_B     = Team B action
A_A(s)  = Team A legal actions
A_B(s)  = Team B legal actions
```

This formulation provides the complete mathematical foundation for the tug-of-war DP implementation.