# Complexity Analysis

## State Space Complexity

### Basic State
```text
S_t = (t, x_1, x_2, ..., x_R)
```

### Number of States
```text
|S| = T × (2M + 1)^R
```

where:
- T = number of time steps
- M = maximum rope position (range is -M to +M)
- R = number of ropes

### Example Calculations

#### Small Game (3v3, 2 ropes, 10 turns, M=5)
```text
|S| = 10 × 11² = 1,210 states
```

#### Medium Game (6v6, 3 ropes, 20 turns, M=10)
```text
|S| = 20 × 21³ = 185,220 states
```

#### Large Game (12v12, 3 ropes, 60 turns, M=10)
```text
|S| = 60 × 21³ = 555,660 states
```

## Action Space Complexity

### Number of Allocations
```text
|A| = C(P + R - 1, R - 1)
```

This is the stars-and-bars problem: distributing P identical items (players) into R distinct bins (ropes).

### Example Calculations

#### 3 players, 2 ropes
```text
|A| = C(3 + 2 - 1, 2 - 1) = C(4,1) = 4 allocations
```

#### 6 players, 3 ropes
```text
|A| = C(6 + 3 - 1, 3 - 1) = C(8,2) = 28 allocations
```

#### 12 players, 3 ropes
```text
|A| = C(12 + 3 - 1, 3 - 1) = C(14,2) = 91 allocations
```

## Joint Action Space (Adversarial)

### Number of Joint Actions
```text
|A_A × A_B| = |A|²
```

### Example Calculations

#### 3v3, 2 ropes
```text
|A_A × A_B| = 4² = 16 joint actions
```

#### 12v12, 3 ropes
```text
|A_A × A_B| = 91² = 8,281 joint actions
```

## Time Complexity

### Basic Minimax DP
```text
Time = O(T × (2M + 1)^R × C(P + R - 1, R - 1)²)
```

### Derivation
- For each state: evaluate all joint actions
- For each joint action: recurse to next state
- Number of states: T × (2M + 1)^R
- Joint actions per state: C(P + R - 1, R - 1)²

### Example Calculations

#### Small Game (3v3, 2 ropes, 10 turns, M=5)
```text
Time = O(10 × 11² × 4²) = O(1,936) operations
```

#### Medium Game (6v6, 3 ropes, 20 turns, M=10)
```text
Time = O(20 × 21³ × 28²) = O(20 × 9,261 × 784) = O(145,338,080) operations
```

#### Large Game (12v12, 3 ropes, 60 turns, M=10)
```text
Time = O(60 × 21³ × 91²) = O(60 × 9,261 × 8,281) = O(4,603,484,060) operations
```

## Space Complexity

### With Memoization Only
```text
Space = O(T × (2M + 1)^R)
```

### With Policy Storage
```text
Space = O(T × (2M + 1)^R × log|A|)
```

### Example Calculations

#### Small Game
```text
Space = O(10 × 11²) = O(1,210) states
```

#### Large Game
```text
Space = O(60 × 21³) = O(555,660) states
```

## Fatigue-Aware Complexity

### Extended State
```text
S_t = (t, x_1, ..., x_R, E_A, E_B)
```

### Energy Discretization
```text
E_compressed = floor(E / E_max × buckets)
```

### Number of Energy States
```text
|E| = buckets²
```

### Total States with Fatigue
```text
|S| = T × (2M + 1)^R × buckets²
```

### Example (20 energy buckets)
```text
Small game: 1,210 × 400 = 484,000 states
Large game: 555,660 × 400 = 222,264,000 states
```

## Heterogeneous Players Complexity

### Individual Tracking (Infeasible)
```text
S_t = (t, x_1, ..., x_R, s_1, ..., s_{2P})
```

### Number of Individual Stamina States
```text
|S_stamina| = (S_max / Δs + 1)^{2P}
```

For S_max=100, Δs=1, P=12:
```text
|S_stamina| = 101²⁴ ≈ 10⁸ states per time step
```

### Class-Based Aggregation
```text
S_t = (t, x_1, ..., x_R, E_strong, E_normal, E_weak)
```

### Number of Class States
```text
|S_class| = buckets³
```

For 20 buckets:
```text
|S_class| = 8,000 states per time step
```

## Switching Constraints Complexity

### State with Assignments
```text
S_t = (t, x_1, ..., x_R, E_A, E_B, assignments_A, assignments_B)
```

### Number of Assignment States
```text
|assignments| = C(P + R - 1, R - 1)
```

### Total States with Assignments
```text
|S| = T × (2M + 1)^R × buckets² × C(P + R - 1, R - 1)²
```

### Example (12v12, 3 ropes)
```text
|S| = 60 × 21³ × 20² × 91² ≈ 1.3×10⁹ states
```

## Information Model Complexity

### Perfect Information
```text
Observation = S_t
State = S_t
No additional complexity
```

### Local Information
```text
Observation = subset of S_t
State = S_t (reality unchanged)
Strategy complexity: No change
```

### Partial Observability
```text
Belief state B_t = probability distribution over S_t
|B| = 2^{|S|} (exponential in state size)
```

## Approximate Method Complexity

### Limited Horizon H
```text
States evaluated: (2M + 1)^R × |A|² × H
Instead of: T × (2M + 1)^R × |A|²
```

Reduction factor: T/H

### Beam Search with Width K
```text
States evaluated: K × (2M + 1)^R × |A|² × H
Instead of: T × (2M + 1)^R × |A|²
```

Reduction factor: T × (2M + 1)^R / K

### Monte Carlo Tree Search
```text
States evaluated: iterations × average_depth
Tradeoff: Quality vs computation time
```

## Comparative Complexity Table

| Configuration | States | Actions | Time Complexity | Space Complexity | Exact DP |
|--------------|-------|---------|----------------|------------------|----------|
| 3v3, 2 ropes, 10 turns | 1,210 | 4 | ~2,000 | ~1,200 | ✓ |
| 6v6, 3 ropes, 20 turns | 185,220 | 28 | ~145M | ~185K | ⚠️ |
| 12v12, 3 ropes, 60 turns | 555,660 | 91 | ~4.6B | ~556K | ❌ |
| + fatigue (20 buckets) | 222M | 91 | ~18B | ~222M | ❌ |
| + individual stamina | 10¹⁴ | 91 | ~10²⁰ | ~10¹⁴ | ❌ |
| + switching | 1.3×10⁹ | 2 | ~10¹⁰ | ~1.3×10⁹ | ❌ |
| + assignments | 1.3×10⁹ | 91 | ~10¹¹ | ~1.3×10⁹ | ❌ |

## Complexity Growth Rate

### Exponential in Ropes
```text
|S| ∝ (2M + 1)^R
```

Adding one rope multiplies state space by ~2M+1

### Polynomial in Players
```text
|A| = C(P + R - 1, R - 1) ≈ P^{R-1} / (R-1)!
```

For fixed R, grows polynomially in P

### Quadratic in Joint Actions
```text
|A_A × A_B| = |A|²
```

Two-player adversarial requirement

### Linear in Time
```text
|S| ∝ T
```

## Feasibility Thresholds

### Recommended Limits for Exact DP

#### Small Experiments
```text
P ≤ 4, R ≤ 2, T ≤ 10, M ≤ 5
Estimated states: ~10³
Estimated transitions: ~10⁴
```

#### Medium Experiments
```text
P ≤ 6, R ≤ 3, T ≤ 20, M ≤ 10
Estimated states: ~10⁵
Estimated transitions: ~10⁸
```

#### Large Experiments (Use Approximate)
```text
P = 12, R = 3, T = 60, M = 10
Estimated states: ~10⁶
Estimated transitions: ~10¹⁰
```

## Memory Requirements

### State Storage
```text
sizeof(StateKey) ≈ 8 + 4R bytes
```

For R=3: ~20 bytes per state

### Value Storage
```text
sizeof(f64) = 8 bytes per state
```

### Policy Storage
```text
sizeof(Allocation) ≈ 8R bytes per state
```

### Total Memory Examples

#### Small Game (1,210 states)
```text
Values: 1,210 × 8 = 9.7 KB
Policies: 1,210 × 24 = 29 KB
Total: ~40 KB
```

#### Large Game (555,660 states)
```text
Values: 555,660 × 8 = 4.4 MB
Policies: 555,660 × 24 = 13.3 MB
Total: ~18 MB
```

## Computation Time Estimates

### Single Operation Cost
```text
Force calculation: O(P) per rope
Position update: O(1) per rope
Stamina update: O(P)
```

### Per State Cost
```text
O(R × P + P) = O(P × (R + 1))
```

### Total Cost
```text
Time = |S| × |A|² × O(P × (R + 1))
```

### Example (12v12, 3 ropes)
```text
Per state: O(12 × 4) = 48 operations
Transitions: 4.6B × 48 ≈ 2.2×10¹¹ operations
```

## Optimization Strategies

### Alpha-Beta Pruning
Expected reduction: 10-100× depending on game balance

### Action Pruning
Expected reduction: 2-10× by eliminating dominated actions

### State Compression
Expected reduction: 10-1000× depending on compression scheme

### Memoization Hit Rate
Typical: 80-95% of states visited multiple times

## Scalability Limits

### Practical Limits
```text
Exact DP feasible when:
    T × (2M + 1)^R × C(P + R - 1, R - 1)² < 10⁸
```

### Beyond Limits
```text
Use:
    - Limited horizon planning
    - Beam search
    - Monte Carlo methods
    - Heuristic strategies
```

This analysis shows why exact DP works for small configurations but requires approximation for realistic games.