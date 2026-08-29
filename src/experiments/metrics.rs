use crate::experiments::ExperimentResults;
use std::fmt;

impl fmt::Display for ExperimentResults {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "Experiment Results Summary")?;
        writeln!(f, "==========================")?;
        writeln!(f, "Total Games: {}", self.total_games)?;
        writeln!(f, "Team A Wins: {} ({:.1}%)", self.wins_a, self.win_rate_a() * 100.0)?;
        writeln!(f, "Team B Wins: {} ({:.1}%)", self.wins_b, self.win_rate_b() * 100.0)?;
        writeln!(f, "Draws: {} ({:.1}%)", self.draws, self.draw_rate() * 100.0)?;
        writeln!(f, "Average Score A: {:.2}", self.average_score_a)?;
        writeln!(f, "Average Score B: {:.2}", self.average_score_b)?;
        writeln!(f, "Average Score Differential: {:.2}", self.average_score_differential)?;
        writeln!(f, "Average Energy A: {:.1}", self.average_energy_a)?;
        writeln!(f, "Average Energy B: {:.1}", self.average_energy_b)
    }
}

/// Calculate confidence interval for win rate
pub fn calculate_win_rate_confidence_interval(wins: usize, total: usize, confidence: f64) -> (f64, f64) {
    if total == 0 {
        return (0.0, 0.0);
    }
    
    let p = wins as f64 / total as f64;
    let z = match confidence {
        0.90 => 1.645,
        0.95 => 1.96,
        0.99 => 2.576,
        _ => 1.96, // Default to 95%
    };
    
    let margin = z * (p * (1.0 - p) / total as f64).sqrt();
    let lower = (p - margin).max(0.0);
    let upper = (p + margin).min(1.0);
    
    (lower, upper)
}

/// Format a comparison table for multiple strategies
pub fn format_comparison_table(results: &[(String, ExperimentResults)]) -> Result<String, String> {
    let mut output = String::new();
    
    // Header
    output.push_str("Strategy Comparison\n");
    output.push_str("====================\n");
    output.push_str(&format!("{:<20} {:>8} {:>8} {:>8} {:>8} {:>10}\n", 
        "Strategy", "Wins", "Losses", "Draws", "Win Rate", "Avg Score"));
    output.push_str(&format!("{:<20} {:>8} {:>8} {:>8} {:>8} {:>10}\n", 
        "--------", "----", "------", "-----", "--------", "---------"));
    
    for (strategy_name, result) in results {
        let win_rate = result.win_rate_a() * 100.0;
        let losses = result.wins_b; // Assuming this is for Team A
        
        output.push_str(&format!("{:<20} {:>8} {:>8} {:>8} {:>7.1} {:>10.2}\n",
            strategy_name,
            result.wins_a,
            losses,
            result.draws,
            win_rate,
            result.average_score_differential
        ));
    }
    
    Ok(output)
}