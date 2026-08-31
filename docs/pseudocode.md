# Pseudocode for Tug-of-War DP

## Basic Minimax DP Solver

### High-Level Algorithm
```text
FUNCTION Solve(state):
    IF state.time == T:
        RETURN terminal_value(state)
    
    IF state exists in memo:
        RETURN memo[state]
    
    best_value = -∞
    
    FOR each action_A in legal_actions_A(state):
        worst_value = +∞
        
        FOR each action_B in legal_actions_B(state):
            next_state = transition(state, action_A, action_B)
            value = Solve(next_state)
            worst_value = MIN(worst_value, value)
        
        best_value = MAX(best_value, worst_value)
    
    memo[state] = best_value
    RETURN best_value
```

### Policy Extraction
```text
FUNCTION BestAction(state):
    best_action = None
    best_value = -∞
    
    FOR each action_A in legal_actions_A(state):
        worst_value = +∞
        
        FOR each action_B in legal_actions_B(state):
            next_state = transition(state, action_A, action_B)
            value = Solve(next_state)
            worst_value = MIN(worst_value, value)
        
        IF worst_value > best_value:
            best_value = worst_value
            best_action = action_A
    
    RETURN best_action
```

## Allocation Generation

### Stars and Bars Algorithm
```text
FUNCTION GenerateAllocations(P, R):
    allocations = []
    current = [0] * R
    
    FUNCTION Generate(index, remaining):
        IF index == R - 1:
            current[index] = remaining
            ADD current to allocations
            RETURN
        
        FOR count in 0 to remaining:
            current[index] = count
            Generate(index + 1, remaining - count)
    
    Generate(0, P)
    RETURN allocations
```

### Number of Allocations
```text
FUNCTION CountAllocations(P, R):
    RETURN Binomial(P + R - 1, R - 1)
```

## Fatigue-Aware DP

### State Compression
```text
FUNCTION CompressEnergy(energy, max_energy, buckets):
    ratio = energy / max_energy
    ratio = CLAMP(ratio, 0, 1)
    bucket = ROUND(ratio × (buckets - 1))
    RETURN CLAMP(bucket, 0, buckets - 1)
```

### Extended State Key
```text
state_key = (time, rope_positions, energy_A, energy_B)
```

### Fatigue-Aware Solve
```text
FUNCTION SolveFatigue(state):
    energy_A = CompressEnergy(state.energy_A, state.max_energy_A, buckets)
    energy_B = CompressEnergy(state.energy_B, state.max_energy_B, buckets)
    
    key = (state.time, state.positions, energy_A, energy_B)
    
    IF key in cache:
        RETURN cache[key]
    
    // ... rest same as basic DP ...
```

## Limited Horizon Approximation

### Beam Search
```text
FUNCTION SolveLimitedHorizon(state, horizon, beam_width):
    beam = [(state, 0.0)]
    
    FOR step in 1 to horizon:
        next_beam = []
        
        FOR (current_state, current_value) in beam:
            FOR each action in legal_actions(current_state):
                next_state = transition(current_state, action)
                opponent_action = GetOpponentResponse(next_state)
                next_state = transition(next_state, opponent_action)
                value = Evaluate(next_state) + current_value
                ADD (next_state, value) to next_beam
        
        SORT next_beam by value descending
        beam = first beam_width elements of next_beam
    
    RETURN max value in beam
```

### Best Action Selection
```text
FUNCTION GetBestActionLimited(state):
    best_action = None
    best_value = -∞
    
    FOR each action in legal_actions(state):
        next_state = transition(state, action)
        value = SolveLimitedHorizon(next_state, horizon, beam_width)
        
        IF value > best_value:
            best_value = value
            best_action = action
    
    RETURN best_action
```

## Experiment Runner

### Parallel Experiment Execution
```text
FUNCTION RunExperiments(config):
    results = []
    
    FOR thread in parallel_threads:
        FOR game in thread_games:
            state = InitializeGame(config, game.seed)
            strategy_A = CreateStrategy(config.team_a_strategy, game.seed)
            strategy_b = CreateStrategy(config.team_b_strategy, game.seed + 1)
            
            WHILE NOT state.finished():
                action_a = strategy_a.decide(state, Team_A)
                action_b = strategy_b.decide(state, Team_B)
                state = step(state, action_a, action_b)
            
            score = calculate_score(state)
            ADD (score, state.energy) to results
    
    RETURN AggregateStatistics(results)
```

## Bottom-Up DP

### Tabulation Form
```text
FUNCTION SolveBottomUp():
    // Initialize terminal states
    FOR each state where time == T:
        dp[state] = terminal_value(state)
    
    // Iterate backwards through time
    FOR t from T-1 down to 0:
        FOR each state where time == t:
            dp[state] = -∞
            
            FOR each action_A in legal_actions_A(state):
                worst_value = +∞
                
                FOR each action_B in legal_actions_B(state):
                    next_state = transition(state, action_A, action_B)
                    worst_value = MIN(worst_value, dp[next_state])
                
                dp[state] = MAX(dp[state], worst_value)
    
    RETURN dp[initial_state]
```

## Alpha-Beta Pruning

### Alpha-Beta Minimax
```text
FUNCTION AlphaBeta(state, alpha, beta):
    IF state.time == T:
        RETURN terminal_value(state)
    
    IF state in cache:
        RETURN cache[state]
    
    best_value = -∞
    
    FOR each action_A in legal_actions_A(state):
        worst_value = +∞
        
        FOR each action_B in legal_actions_B(state):
            value = AlphaBeta(transition(state, action_A, action_B), alpha, beta)
            worst_value = MIN(worst_value, value)
            
            IF worst_value <= alpha:
                BREAK  // Beta cutoff
        
        best_value = MAX(best_value, worst_value)
        
        IF best_value >= beta:
            BREAK  // Alpha cutoff
    
    cache[state] = best_value
    RETURN best_value
```

## Monte Carlo Tree Search (Future)

### MCTS Structure
```text
FUNCTION MCTS(state, iterations):
    root = CreateNode(state)
    
    FOR iteration in 1 to iterations:
        node = root
        
        // Selection
        WHILE node is fully expanded:
            node = SelectChild(node, UCB1)
        
        // Expansion
        IF NOT node is terminal:
            child = ExpandRandom(node)
            node = child
        
        // Simulation
        result = SimulateRandom(node.state)
        
        // Backpropagation
        WHILE node != root:
            UpdateNode(node, result)
            node = node.parent
    
    RETURN MostVisitedChild(root).action
```

## Switching Constraints

### Switch-Aware Action Generation
```text
FUNCTION LegalActionsSwitching(state, team):
    current_allocation = GetCurrentAllocation(state, team)
    possible_actions = []
    
    FOR each allocation in all_allocations:
        switches = CountSwitches(current_allocation, allocation)
        
        IF switches <= max_switches:
            ADD allocation to possible_actions
    
    RETURN possible_actions
```

## Commander System

### Command Execution
```text
FUNCTION ExecuteCommands(state):
    current_time = state.time
    remaining_commands = []
    
    FOR (execute_time, player, rope) in state.pending_commands:
        IF execute_time <= current_time:
            state.assignments[player] = rope
        ELSE:
            ADD (execute_time, player, rope) to remaining_commands
    
    state.pending_commands = remaining_commands
```

### Command Issuance
```text
FUNCTION IssueCommand(state, team, player, rope):
    IF NOT CanIssueCommand(state, team):
        RETURN False
    
    execute_time = state.time + command_delay
    ADD (execute_time, player, rope) to state.pending_commands
    state.last_command_time = state.time
    RETURN True
```

## Information Models

### Observation Generation
```text
FUNCTION GetObservation(state, team, model):
    SWITCH model:
        CASE PerfectInformation:
            RETURN (all rope positions, all forces, all allocations)
        
        CASE LocalInformation:
            current_rope = GetCurrentRope(team)
            RETURN (current_rope position, current_rope force, current_rope allocation)
        
        CASE DelayedInformation(delay):
            effective_time = MAX(0, state.time - delay)
            RETURN HistoricalObservation(effective_time)
```

## Spy System

### Spy Behavior Application
```text
FUNCTION ApplySpyBehavior(state, spy_config):
    FOR each spy in state.spies:
        IF spy.is_hidden AND ShouldBetray(spy, state):
            worst_rope = FindWorstRopeForTeam(state, spy.team)
            MovePlayer(spy.id, worst_rope)
```

### Adaptive Betrayal Decision
```text
FUNCTION ShouldBetray(spy, state):
    score = CalculateScore(state)
    advantage = score[spy.team] - score[opponent]
    
    IF advantage > 0:
        RETURN Random() < 0.7  // Betray when winning
    ELSE:
        RETURN Random() < 0.3  // Less likely when losing
```

## Performance Optimizations

### Action Pruning
```text
FUNCTION PruneDominatedActions(actions, state):
    pruned = []
    
    FOR each action in actions:
        IF NOT IsDominated(action, pruned, state):
            ADD action to pruned
    
    RETURN pruned
```

### State Caching
```text
FUNCTION CacheState(state, value):
    IF cache.size > MAX_CACHE_SIZE:
        EvictOldestEntries()
    
    cache[state] = value
```

This pseudocode corresponds directly to the mathematical formulations and Rust implementations in the project.