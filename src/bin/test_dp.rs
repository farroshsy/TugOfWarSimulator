use tug_of_war::game::{create_small_game};
use tug_of_war::algorithms::{DpSolver, DpFeasibilityEstimator};
use tug_of_war::strategies::Strategy;

fn main() {
    println!("=== DP Solver Test ===\n");
    
    // Test with small game
    let state = create_small_game();
    
    println!("Game configuration:");
    println!("People per team: {}", state.get_team_player_count(tug_of_war::game::Team::A));
    println!("Ropes: {}", state.ropes.len());
    println!("Duration: {}", state.rules.duration);
    println!("Position range: {}", state.rules.position_range);
    
    // Check feasibility
    let estimator = DpFeasibilityEstimator::new(
        state.get_team_player_count(tug_of_war::game::Team::A),
        state.ropes.len(),
        state.rules.duration,
        state.rules.position_range,
    );
    
    println!("\n{}", estimator.report());
    
    // Solve the game
    println!("\nSolving DP...");
    let mut solver = DpSolver::new(
        state.get_team_player_count(tug_of_war::game::Team::A),
        state.ropes.len(),
    );
    
    let start = std::time::Instant::now();
    let value = solver.solve(&state);
    let duration = start.elapsed();
    
    println!("DP Value: {:.2}", value);
    println!("Solving time: {:?}", duration);
    println!("Cache size: {} states", solver.cache_stats().0);
    println!("Policy cache size: {} states", solver.cache_stats().1);
    
    // Test getting optimal action
    println!("\nTesting optimal action extraction...");
    if let Some(allocation) = solver.get_optimal_action(&state) {
        println!("Optimal allocation: {:?}", allocation.counts);
    } else {
        println!("No optimal action found");
    }
    
    println!("\n=== Test completed successfully ===");
}