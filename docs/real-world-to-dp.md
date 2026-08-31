# From Real-World Problems to Dynamic Programming

## Overview

This document establishes a reusable methodology for systematically transforming complicated real-world problems into Dynamic Programming formulations. The tug-of-war simulator serves as the primary running example throughout.

## The Four-Layer Hierarchy

```text
Physical Reality
      │
      ▼
Mathematical Model
      │
      ▼
Optimization Problem
      │
      ▼
Algorithm
      │
      ▼
Program
```

Each layer introduces potential errors:
- **Reality → Model**: Abstraction error
- **Model → Algorithm**: Formulation error  
- **Algorithm → Implementation**: Coding error
- **Implementation → Experiment**: Validation error

## Core Principle

> **Do not start by asking "How do I solve this with DP?"**
> 
> Start by asking: **"What information about the past is actually necessary to make the next decision?"**
> 
> That answer is the beginning of the state.

## The Fundamental Question

> **Can I Turn This Real-World Problem Into DP?**

### Question 1 — What decisions can I make?

**Actions**: What can the agent control?

In tug-of-war:
- Which rope should each player pull?
- How many players should go to each rope?
- Should a player rest?
- Should a player switch ropes?
- Should the commander issue a command?

### Question 2 — What changes over time?

**State Variables**: What information changes?

In tug-of-war:
- Rope positions
- Player stamina
- Time remaining
- Score
- Player locations
- Information available
- Command cooldown status

### Question 3 — What information from the past actually matters?

**State Sufficiency**: Does the state contain enough information to determine future possibilities?

For DP to work, the future must depend on the past only through the current state:

```text
History
H_0 ... H_t
       │
       ▼
     S_t
       │
       ▼
Future possibilities
```

### Question 4 — What constraints exist?

**Constraints**: What limits the actions?

In tug-of-war:
- Limited number of players
- Limited stamina
- Limited time
- Switching limits
- Rope weight resistance
- Communication delay
- Information restrictions

## State Construction Method

### Step 1 — List everything that changes
```text
time
rope positions
player stamina
player assignment
score
commander cooldown
pending commands
```

### Step 2 — Remove variables that can be derived
```text
team_force can be calculated from:
  player assignment + player stamina + player strength
```

### Step 3 — Identify variables that affect future decisions
Ask: "If I remove this variable, does it change which action is optimal?"

### Step 4 — Test sufficiency
For each variable, test whether removing it changes future behavior.

### Step 5 — Discretize continuous variables when necessary
```text
position = 3.728 m → position bucket = round(3.728 / Δx)
energy = 74.231 → energy bucket = floor(74.231 / ΔE)
```

## State Representation

### Generic DP State
```text
S_t = (x_1, x_2, ..., x_n)
```

### Tug-of-War State Evolution

**Basic**:
```text
S_t = (t, x_1, x_2, x_3)
```
where `x_i` = position of rope i

**With fatigue**:
```text
S_t = (t, x_1, x_2, x_3, E_A, E_B)
```
where `E_A`, `E_B` = team energy

**Full player-level**:
```text
S_t = (t, rope_positions, player_stamina, player_assignments, score, command_state)
```

### State Compression

Instead of:
```text
stamina_player_1, stamina_player_2, ..., stamina_player_12
```

Use:
```text
team_energy
```

Or discretized buckets:
```text
high_stamina_count (70-100)
medium_stamina_count (40-69)
low_stamina_count (0-39)
```

**Tradeoff**: More information → more accurate model → larger state space → harder computation

## Actions

### Action Space Definition
```text
A(s) = set of legal actions available in state s
```

### Tug-of-War Actions
For a team with P players and R ropes:
```text
action = (a_1, a_2, ..., a_R)
```
where `a_1 + a_2 + ... + a_R = P`

### Number of Allocations
```text
C(P + R - 1, R - 1)
```

This is the stars-and-bars problem.

**Example**: P=12, R=3 → C(14,2) = 91 allocations

## Transition Function

```text
T(s, a) → s'
```

where:
- `s` = current state
- `a` = action  
- `s'` = next state

### Tug-of-War Transition
```text
s = (t, x_1, x_2, x_3, energyA, energyB)
a = (allocationA, allocationB)
s' = T(s, a)
```

The transition:
1. Calculates pulling forces
2. Calculates rope movement
3. Updates positions
4. Updates stamina
5. Applies recovery
6. Applies switching cost
7. Updates time
8. Updates other state variables

## Objective Function

### Terminal Condition
```text
V_T(s) = final_score(s)
```

### Tug-of-War Reward
```text
Reward(s_T) = Team A final score - Team B final score
```

```text
V_T(s) = Σ value_i (if rope_i belongs to Team A)
         - Σ value_i (if rope_i belongs to Team B)
```

## Why This Is Dynamic Programming

### Optimal Substructure
An optimal solution from the current state can be expressed using optimal solutions to future subproblems.

### Overlapping Subproblems
Different action sequences can lead to the same state:

```text
History A ─────┐
               ├──→ S_t ─→ same future problem
History B ─────┘
```

Once we reach the same `S_t`, we do not need to solve the future twice.

## Bellman Equation

### Single-Agent Finite-Horizon
```text
V_t(s) = max_{a ∈ A(s)} [R(s,a) + V_{t+1}(T(s,a))]
```

### Two-Player Adversarial (Tug-of-War)
```text
V_t(s) = max_{a_A ∈ A_A(s)} min_{a_B ∈ A_B(s)} [R(s,a_A,a_B) + V_{t+1}(T(s,a_A,a_B))]
```

This is **finite-horizon minimax dynamic programming**, not ordinary single-agent DP.

## Complexity Analysis

### State Count
```text
number of states ≈ T × D^R
```
where:
- T = number of time steps
- D = number of possible positions per rope
- R = number of ropes

### Action Count
```text
A = C(P + R - 1, R - 1)
```

### Joint Actions (Adversarial)
```text
joint actions ≈ A²
```

### Time Complexity
```text
Time ≈ O(T × D^R × A²)
```

### Substituting A
```text
Time ≈ O(T × D^R × C(P + R - 1, R - 1)²)
```

### Space Complexity
```text
Space ≈ O(T × D^R)
```

## State Explosion Case Study

| Model | State Variables | States | Actions | Exact DP |
|-------|----------------|-------|---------|----------|
| Basic | position | ~1,000 | 91 | feasible |
| + fatigue | energy | ~400,000 | 91 | difficult |
| + players | individual state | ~10^12 | 91 | infeasible |
| + commands | command queue | ~10^15 | 91 | very difficult |
| + spies | hidden state | ~10^18 | 91 | different problem |

## Exact vs Approximate Model

### Exact DP
- Solves the mathematical model exactly
- Provides ground-truth optimal policy
- Only feasible for small state spaces

### Approximate Methods
- **State aggregation**: Compress state representation
- **Coarse discretization**: Larger buckets for continuous variables
- **Limited horizon**: Plan only N steps ahead
- **Beam search**: Keep only top K states
- **Action pruning**: Eliminate dominated actions
- **Value approximation**: Approximate value function
- **Rollouts**: Monte Carlo simulation
- **MCTS**: Monte Carlo Tree Search

### Approximation Tradeoffs
For each approximation:
- What information is discarded?
- Why does it reduce complexity?
- What error might it introduce?
- How can we measure that error?

## Decision Tree: Is This Actually DP?

```text
Does the problem involve sequential decisions?
        │
        ├── NO → probably not DP
        │
        ▼
Does the future depend on previous decisions?
        │
        ├── NO → may be simpler optimization
        │
        ▼
Can the relevant history be compressed into a state?
        │
        ├── NO → DP may not apply directly
        │
        ▼
Do different histories reach the same state?
        │
        ├── NO → brute-force recursion may suffice
        │
        ▼
Does the problem have optimal substructure?
        │
        ├── NO → ordinary DP may not work
        │
        ▼
Can the state/action space be bounded?
        │
        ├── NO → need approximation / different method
        │
        ▼
DYNAMIC PROGRAMMING CANDIDATE
```

## Distinguishing DP From Other Algorithms

- **Single agent + sequential decisions** → DP
- **Two adversarial agents** → Minimax DP / game-tree methods
- **Random transitions** → Stochastic DP / MDP
- **Unknown state** → POMDP / belief-state methods
- **Very large state space** → Approximate DP / RL / MCTS
- **No sequential dependency** → greedy / matching / flow / ILP
- **Continuous state/action** → continuous optimization / control methods

## The Model Ladder

```text
LEVEL 0: 1 rope, 1 player/team, deterministic
    ↓
LEVEL 1: multiple ropes
    ↓
LEVEL 2: multiple players
    ↓
LEVEL 3: fatigue
    ↓
LEVEL 4: player heterogeneity
    ↓
LEVEL 5: switching constraints
    ↓
LEVEL 6: communication delay
    ↓
LEVEL 7: partial information
    ↓
LEVEL 8: spies / stochastic behavior
    ↓
REALISTIC GAME
```

For each level:
- New variable
- New state dimension
- New constraint
- New transition
- New complexity
- New algorithmic challenge

## Feature → Mathematical Consequence Mapping

| Real-world feature | Mathematical consequence |
|-------------------|----------------------|
| More ropes | Larger state/action space |
| More players | More allocation combinations |
| Fatigue | Energy becomes state |
| Different strengths | Player identity matters |
| Switching | Action constraints |
| Switching cost | Transition cost |
| Commander | Communication state |
| Command delay | Delayed transition |
| Spy | Hidden state / adversarial uncertainty |
| Random fatigue | Stochastic transition |
| Unknown opponent | Partial observability |
| Continuous position | Discretization |
| Longer game | Longer horizon |

## Final Mental Model

```text
             REAL WORLD
                 │
                 ▼
        ┌─────────────────┐
        │ What can change?│
        └────────┬────────┘
                 ▼
        ┌─────────────────┐
        │ What can I      │
        │ control?        │
        └────────┬────────┘
                 ▼
        ┌─────────────────┐
        │ What past       │
        │ information     │
        │ matters?        │
        └────────┬────────┘
                 ▼
        ┌─────────────────┐
        │      STATE      │
        └────────┬────────┘
                 ▼
        ┌─────────────────┐
        │     ACTIONS     │
        └────────┬────────┘
                 ▼
        ┌─────────────────┐
        │   TRANSITION    │
        └────────┬────────┘
                 ▼
        ┌─────────────────┐
        │     REWARD      │
        └────────┬────────┘
                 ▼
        ┌─────────────────┐
        │   SUBPROBLEM    │
        └────────┬────────┘
                 ▼
        ┌─────────────────┐
        │ BELLMAN / DP    │
        └────────┬────────┘
                 ▼
        ┌─────────────────┐
        │ ALGORITHM        │
        └────────┬────────┘
                 ▼
        ┌─────────────────┐
        │     PROGRAM     │
        └────────┬────────┘
                 ▼
        ┌─────────────────┐
        │   EXPERIMENT    │
        └────────┬────────┘
                 │
                 └──────→ Does the model
                          actually represent
                          the real world?
```

## Key Insight

> **Choosing a DP state is equivalent to making a claim about which aspects of history matter.**
>
> If we define `S = rope positions`, we are claiming: "Once rope positions are known, everything else about the past is irrelevant to the future."
>
> If that claim is false, the DP model is wrong.

## Conclusion

The ultimate objective is for this project to become not just a tug-of-war simulator, but a **worked example of how computational thinking turns messy real-world systems into formally solvable subproblems**.

When you encounter a new real-world problem, return to this framework and ask systematically: "What information do I actually need to make the next decision?" The answer to that question is the beginning of your DP state.