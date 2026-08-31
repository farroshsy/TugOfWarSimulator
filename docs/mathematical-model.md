# Mathematical Model of Tug-of-War

## Problem Definition

Two teams compete over multiple weighted ropes. Each rope has a position, weight, and point value. Players can choose which rope to pull. Players become fatigued, can switch ropes, and may receive delayed commands. The game lasts for a finite amount of time. At the end, teams receive points according to which territory contains each rope.

## Mathematical Notation

### Core Variables
- `t` = current timestep (0 ≤ t ≤ T)
- `T` = terminal timestep (game duration)
- `R` = number of ropes
- `P` = number of players per team
- `i` = rope index (1 ≤ i ≤ R)
- `j` = player index (1 ≤ j ≤ P)

### Rope Properties
- `x_i(t)` = position of rope i at time t
- `w_i` = weight of rope i
- `v_i` = point value of rope i
- `x_i < 0` = rope in Team A territory
- `x_i > 0` = rope in Team B territory
- `x_i = 0` = rope at center

### Player Properties
- `s_j(t)` = stamina of player j at time t
- `S_max` = maximum stamina
- `σ_j` = strength of player j
- `f_j` = fatigue rate of player j
- `r_j` = recovery rate of player j

### Effective Strength
```text
effective_strength_j(t) = σ_j × (s_j(t) / S_max)
```

### Rope Movement
```text
F_i(t) = Σ_{j∈A_i} effective_strength_j(t) - Σ_{j∈B_i} effective_strength_j(t)
```

where `A_i` = Team A players on rope i, `B_i` = Team B players on rope i

### Movement Model
```text
Δx_i(t) = 
    0 if |F_i(t)| ≤ w_i
    sign(F_i(t)) if |F_i(t)| > w_i
```

### Position Update
```text
x_i(t+1) = x_i(t) + Δx_i(t)
```

### Stamina Update
```text
s_j(t+1) = 
    s_j(t) - f_j if pulling
    s_j(t) + r_j if resting
    clamped to [0, S_max]
```

## State Definition

### Basic State
```text
S_t = (t, x_1, x_2, ..., x_R)
```

### With Fatigue
```text
S_t = (t, x_1, x_2, ..., x_R, E_A, E_B)
```
where `E_A = Σ_j s_j` for Team A, `E_B = Σ_j s_j` for Team B

### Full State
```text
S_t = (t, x_1, ..., x_R, s_1, ..., s_{2P}, a_1, ..., a_{2P}, C_A, C_B)
```
where `a_j` = current rope assignment, `C_A`, `C_B` = command states

## Action Definition

### Allocation Action
```text
action = (a_1, a_2, ..., a_R)
```
where `a_i` = number of players assigned to rope i

### Action Constraints
```text
Σ_{i=1}^R a_i = P
a_i ≥ 0
```

### Number of Allocations
```text
|A| = C(P + R - 1, R - 1)
```

### Switching Constraints
```text
|{j : a_j(t) ≠ a_j(t-1)}| ≤ K
```
where K = maximum switches per turn

## Transition Function

```text
T: S × A_A × A_B → S'
```

### Transition Steps
1. Apply actions: move players according to allocations
2. Apply switching costs: reduce stamina for switchers
3. Calculate forces: compute effective strength for each rope
4. Update positions: apply movement model
5. Update stamina: apply fatigue/recovery
6. Update time: t ← t+1

## Terminal Condition

```text
Terminal: t = T
```

## Reward Function

### Terminal Reward
```text
R_T(S) = Σ_{i: x_i(T) < 0} v_i - Σ_{i: x_i(T) > 0} v_i
```

### Intermediate Reward
```text
R_t(S, a_A, a_B) = 0
```
(No intermediate rewards in basic formulation)

## Value Function

```text
V_t(S) = optimal future score difference from state S at time t
```

## Bellman Recurrence

### Two-Player Adversarial
```text
V_t(S) = max_{a_A ∈ A_A(S)} min_{a_B ∈ A_B(S)} V_{t+1}(T(S, a_A, a_B))
```

### Base Case
```text
V_T(S) = R_T(S)
```

## Optimal Policy

```text
π(S) = argmax_{a_A ∈ A_A(S)} min_{a_B ∈ A_B(S)} V_{t+1}(T(S, a_A, a_B))
```

## Information Model

### Perfect Information
```text
Observation_t = S_t
```

### Local Information
```text
Observation_t = {x_j(t), F_j(t), opponent_allocations_j(t)}
```
where j = current rope of observer

### Delayed Information
```text
Observation_t = S_{t-d}
```
where d = delay

## Game Theory Classification

### Deterministic Perfect Information
```text
Players: 2
Information: Perfect
Transitions: Deterministic
Objective: Zero-sum
```
→ Finite-horizon minimax DP

### Stochastic
```text
Transitions: Random
```
→ Stochastic DP / MDP

### Partial Observability
```text
Information: Imperfect
```
→ POMDP / belief-state methods

### Hidden Adversaries
```text
Hidden spy behavior
```
→ Bayesian game / stochastic game

## Complexity Summary

### State Space Size
```text
|S| = T × (2M + 1)^R
```
where M = maximum rope position

### Action Space Size
```text
|A| = C(P + R - 1, R - 1)
```

### Joint Action Space
```text
|A_A × A_B| = C(P + R - 1, R - 1)²
```

### Time Complexity
```text
O(T × (2M + 1)^R × C(P + R - 1, R - 1)²)
```

### Space Complexity
```text
O(T × (2M + 1)^R)
```

## Example Calculation

For P=12, R=3, M=10, T=60:
- State space: 60 × 21³ = 555,660
- Action space: C(14,2) = 91
- Joint actions: 91² = 8,281
- Transitions: 555,660 × 8,281 ≈ 4.6 billion

This explains why exact DP is only feasible for smaller configurations.