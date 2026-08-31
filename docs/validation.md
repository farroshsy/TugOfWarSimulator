# Validation Methodology

## Overview

Validation ensures that our mathematical model, algorithms, and implementation correctly represent the real-world problem. This document describes systematic validation procedures for the tug-of-war simulator.

## Validation Layers

```text
REALITY
   ↓ validation
MODEL
   ↓ validation
ALGORITHM
   ↓ validation
IMPLEMENTATION
   ↓ validation
EXPERIMENT
```

## Level 1: Model Validation

### Purpose
Ensure the mathematical model accurately represents the real physical game.

### Validation Questions
1. Does the model capture essential physical constraints?
2. Are the simplifications justified?
3. Does the model preserve strategic richness?

### Validation Methods

#### Physical Plausibility
```text
Test: Can heavier ropes still be moved by enough players?
Expected: Yes, if force > weight
Check: Simulate extreme cases
```

#### Strategic Consistency
```text
Test: Does the model allow reasonable strategies?
Expected: Yes, such as "attack high-value rope"
Check: Verify action space includes plausible allocations
```

#### Conservation Laws
```text
Test: Is total player count conserved?
Expected: Σ a_i = P for all allocations
Check: Verify allocation generation
```

## Level 2: Algorithm Validation

### Purpose
Ensure the DP algorithm correctly solves the mathematical model.

### Validation Methods

#### Small Hand-Computed Examples
```text
Configuration: P=1, R=1, T=3, M=1
Manual calculation:
  State 0: x=0
  State 1: x=±1 (depending on action)
  State 2: x=±2
  State 3: x=±3 (terminal)
Verify DP result matches manual calculation
```

#### Recursion vs Memoization
```text
Test: Does memoized DP produce same results as naive recursion?
Method: Compare results on small configurations
Expected: Identical values
```

#### Bottom-Up vs Top-Down
```text
Test: Does tabulation produce same results as memoization?
Method: Implement both and compare
Expected: Identical values
```

#### Policy Consistency
```text
Test: Does extracted policy match value function?
Method: For each state, check if π(s) achieves V(s)
Expected: Policy should achieve the value
```

## Level 3: Implementation Validation

### Unit Tests

#### Allocation Generation
```text
Test: Count allocations = C(P + R - 1, R - 1)
Test: Sum of allocations = P
Test: All allocations are distinct
```

#### Force Calculation
```text
Test: Force with 0 players = 0
Test: Force scales with player strength
Test: Force decreases with fatigue
```

#### Rope Movement
```text
Test: No movement when |F| ≤ weight
Test: Movement toward stronger team
Test: Position bounds respected
```

#### Scoring
```text
Test: Negative position → Team A points
Test: Positive position → Team B points
Test: Zero position → no points
```

### Integration Tests

#### Game Step Consistency
```text
Test: Time advances by 1 each step
Test: Players cannot teleport (switching limits)
Test: Stamina never negative or exceeds maximum
```

#### Strategy Framework
```text
Test: All strategies produce valid actions
Test: Strategies can be swapped mid-game
Test: Random seed produces reproducible results
```

## Level 4: Simulation Validation

### Oracle Experiments

#### Methodology
```text
1. Create small configuration where exact DP is feasible
2. Run exact DP to get optimal policy
3. Test strategies against oracle
4. Measure performance gap
```

#### Example Configuration
```text
P=3, R=2, T=10, M=5
Exact DP feasible: Yes
Strategies to test: Random, Greedy, Reactive, FatigueAware
```

#### Metrics
```text
Win rate vs optimal
Score gap vs optimal
Policy agreement rate
```

## Level 5: Approximate Method Validation

### Validation Methodology
```text
1. Ground truth: Exact DP on small configuration
2. Approximation: Limited horizon/beam search on same config
3. Compare: Value gap, policy disagreement, performance
```

### Acceptable Error Thresholds
```text
Value gap: < 10% of optimal value
Policy disagreement: < 20% of decisions
Performance gap: < 15% win rate difference
```

### Error Analysis
```text
WHEN approximation performs poorly:
- Identify state components causing error
- Increase approximation quality (more buckets, larger horizon)
- Document fundamental limitations
```

## Reproducibility Validation

### Random Seed Testing
```text
Test: Same seed → same results
Method: Run experiment twice with same seed
Expected: Identical final states and scores
```

### Configuration Validation
```text
Test: Same configuration → same strategic possibilities
Method: Vary only random seed
Expected: Different game outcomes but same action space
```

## Statistical Validation

### Significance Testing
```text
For strategy comparison with N games:
- Calculate win rate with confidence interval
- Use Wilson score interval or normal approximation
- Claim "Strategy A better than B" only if intervals don't overlap
```

### Sample Size Determination
```text
For detecting 5% win rate difference with 95% confidence:
N ≈ 1,000 games
For detecting 1% win rate difference:
N ≈ 25,000 games
```

## Performance Profiling

### DP Solver Performance
```text
Measure: Time to solve, cache hit rate, memory usage
Expected: Time grows with state space size
Check: Linear growth in time vs state count
```

### Experiment Runner Performance
```text
Measure: Games per second, thread utilization
Expected: Near-linear scaling with threads
Check: No blocking operations
```

## Edge Case Validation

### Boundary Conditions
```text
Test: t = 0 (initial state)
Test: t = T (terminal state)
Test: All players on one rope
Test: Equal forces (no movement)
Test: Maximum fatigue (zero effective strength)
```

### Error Conditions
```text
Test: Invalid rope indices
Test: Player count mismatch
Test: Negative time/duration
Test: Overflow conditions
```

## Cross-Validation

### Model ↔ Implementation
```text
Verify: Rust structs match mathematical definitions
Check: GameState has all state variables
Check: Transition function matches mathematical definition
```

### Algorithm ↔ Pseudocode
```text
Verify: Rust implementation matches pseudocode
Check: Loop structure matches recurrence
Check: Base cases match terminal conditions
```

## Regression Testing

### Continuous Integration
```text
1. Run all unit tests
2. Run integration tests
3. Run small DP validation
4. Run strategy comparison (small sample)
5. Verify performance within expected bounds
```

### Performance Regression
```text
Alert if: DP solve time increases > 20%
Alert if: Memory usage increases > 50%
Alert if: Win rates change > 10% without code changes
```

## Documentation Validation

### Mathematical Correctness
```text
Verify: Formulas match implementation
Check: Complexity analysis matches measured performance
Check: Examples are correct
```

### Example Correctness
```text
Verify: Small hand-calculated examples are correct
Check: Complexity numbers are accurate
Check: Code examples run successfully
```

## Validation Checklist

### Before Committing Code
- [ ] Unit tests pass
- [ ] Integration tests pass
- [ ] Small DP validation passes
- [ ] No performance regression
- [ ] Documentation updated

### Before Releasing Version
- [ ] All validation tests pass
- [ ] Cross-platform compatibility verified
- [ ] Performance benchmarks within limits
- [ ] Documentation complete and accurate
- [ ] Reproducibility verified

### Before Publishing Results
- [ ] Experiment configuration documented
- [ ] Random seeds recorded
- [ ] Statistical significance verified
- [ ] Code version tagged
- [ ] Results are reproducible

## Failure Mode Analysis

### When Validation Fails

#### Model-Implementation Mismatch
```text
Symptom: DP produces "optimal" but loses in simulation
Diagnosis: Model assumptions violated
Resolution: Adjust model or document limitation
```

#### Algorithm Error
```text
Symptom: DP produces different results than brute force
Diagnosis: Implementation bug in recurrence or memoization
Resolution: Debug and fix algorithm
```

#### State Explosion
```text
Symptom: DP never completes
Diagnosis: State space too large
Resolution: Use approximation methods, document limitation
```

## Validation Documentation

### Test Report Template
```text
Configuration: [details]
Method: [validation approach]
Results: [metrics]
Status: [PASS/FAIL]
Notes: [observations]
```

### Benchmark Report Template
```text
Configuration: [details]
Hardware: [CPU, RAM]
Performance: [time, memory]
Comparison: [vs previous version]
Status: [acceptable/regression]
```

## Conclusion

Validation is not a one-time activity but an ongoing process. Every new feature, optimization, or approximation requires validation against:

1. The mathematical model it claims to solve
2. The algorithm it implements
3. The real-world system it approximates

This systematic validation ensures the tug-of-war simulator serves as a reliable experimental platform for studying DP, game theory, and decision-making under uncertainty.