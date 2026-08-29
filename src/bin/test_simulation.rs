use tug_of_war::game::{create_default_game, create_small_game};
use tug_of_war::game::simulation::{step, calculate_score};
use tug_of_war::strategies::{greedy::GreedyStrategy, reactive::ReactiveStrategy, fatigue::FatigueAwareStrategy, Strategy};

fn main() {
    println!("=== Tug-of-War Simulation Test ===\n");
    
    // Test with small game first
    println!("Testing small game (3v3, 2 ropes, 10 turns)...");
    let mut state = create_small_game();
    
    let mut strategy_a = GreedyStrategy::new();
    let mut strategy_b = ReactiveStrategy::new();
    
    println!("\nInitial state:");
    print_state(&state);
    
    while !state.finished() {
        let action_a = strategy_a.decide(&state, tug_of_war::game::Team::A);
        let action_b = strategy_b.decide(&state, tug_of_war::game::Team::B);
        
        step(&mut state, &action_a, &action_b);
        
        println!("\nAfter step {}:", state.time);
        print_state(&state);
    }
    
    println!("\n=== FINAL RESULT ===");
    let score = calculate_score(&state);
    println!("Team A: {}", score.team_a);
    println!("Team B: {}", score.team_b);
    println!("Score differential: {}", score.differential());
    
    match score.winner() {
        Some(tug_of_war::game::Team::A) => println!("WINNER: Team A"),
        Some(tug_of_war::game::Team::B) => println!("WINNER: Team B"),
        None => println!("DRAW"),
    }
    
    println!("\n=== Testing Fatigue-Aware Strategy ===");
    let mut state2 = create_default_game();
    let mut fatigue_a = FatigueAwareStrategy::new();
    let mut fatigue_b = FatigueAwareStrategy::new();
    
    println!("\nInitial state (default game):");
    print_state(&state2);
    
    // Run a few steps to show fatigue
    for _ in 0..5 {
        if state2.finished() {
            break;
        }
        
        let action_a = fatigue_a.decide(&state2, tug_of_war::game::Team::A);
        let action_b = fatigue_b.decide(&state2, tug_of_war::game::Team::B);
        
        step(&mut state2, &action_a, &action_b);
        
        println!("\nAfter step {}:", state2.time);
        print_state(&state2);
    }
    
    println!("\n=== Test completed successfully ===");
}

fn print_state(state: &tug_of_war::game::GameState) {
    println!("Time: {}/{}", state.time, state.rules.duration);
    
    for rope in &state.ropes {
        println!("  Rope {}: position={}, weight={}, value={}, A players={}, B players={}",
            rope.id, rope.position, rope.weight, rope.value,
            rope.team_a_count, rope.team_b_count);
    }
    
    let score = calculate_score(state);
    println!("  Score: A={}, B={} (diff={})", 
        score.team_a, score.team_b, score.differential());
    
    println!("  Energy: A={:.1}, B={:.1}", 
        state.get_team_energy(tug_of_war::game::Team::A),
        state.get_team_energy(tug_of_war::game::Team::B));
}