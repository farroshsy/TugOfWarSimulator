use crate::experiments::ExperimentResults;
use std::fs::File;
use std::io::Write;
use std::path::Path;

/// Export experiment results to CSV
pub fn export_results_to_csv(results: &ExperimentResults, path: &Path) -> Result<(), String> {
    let mut file = File::create(path)
        .map_err(|e| format!("Failed to create file: {}", e))?;
    
    // Write header
    writeln!(file, "metric,value").map_err(|e| format!("Failed to write header: {}", e))?;
    
    // Write results
    writeln!(file, "total_games,{}", results.total_games).map_err(|e| format!("Failed to write: {}", e))?;
    writeln!(file, "wins_a,{}", results.wins_a).map_err(|e| format!("Failed to write: {}", e))?;
    writeln!(file, "wins_b,{}", results.wins_b).map_err(|e| format!("Failed to write: {}", e))?;
    writeln!(file, "draws,{}", results.draws).map_err(|e| format!("Failed to write: {}", e))?;
    writeln!(file, "win_rate_a,{:.4}", results.win_rate_a()).map_err(|e| format!("Failed to write: {}", e))?;
    writeln!(file, "win_rate_b,{:.4}", results.win_rate_b()).map_err(|e| format!("Failed to write: {}", e))?;
    writeln!(file, "draw_rate,{:.4}", results.draw_rate()).map_err(|e| format!("Failed to write: {}", e))?;
    writeln!(file, "average_score_a,{:.2}", results.average_score_a).map_err(|e| format!("Failed to write: {}", e))?;
    writeln!(file, "average_score_b,{:.2}", results.average_score_b).map_err(|e| format!("Failed to write: {}", e))?;
    writeln!(file, "average_score_differential,{:.2}", results.average_score_differential).map_err(|e| format!("Failed to write: {}", e))?;
    writeln!(file, "average_energy_a,{:.2}", results.average_energy_a).map_err(|e| format!("Failed to write: {}", e))?;
    writeln!(file, "average_energy_b,{:.2}", results.average_energy_b).map_err(|e| format!("Failed to write: {}", e))?;
    
    Ok(())
}

/// Export comparison table to CSV
pub fn export_comparison_to_csv(results: &[(String, ExperimentResults)], path: &Path) -> Result<(), String> {
    let mut file = File::create(path)
        .map_err(|e| format!("Failed to create file: {}", e))?;
    
    // Write header
    writeln!(file, "strategy,wins,losses,draws,win_rate,average_score_differential")
        .map_err(|e| format!("Failed to write header: {}", e))?;
    
    // Write results for each strategy
    for (strategy_name, result) in results {
        let losses = result.wins_b; // Assuming this is for Team A
        writeln!(file, "{},{},{},{},{:.4},{:.2}",
            strategy_name,
            result.wins_a,
            losses,
            result.draws,
            result.win_rate_a(),
            result.average_score_differential
        ).map_err(|e| format!("Failed to write: {}", e))?;
    }
    
    Ok(())
}