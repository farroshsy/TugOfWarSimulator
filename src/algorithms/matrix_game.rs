/// Solution of a two-player zero-sum matrix game.
///
/// `value` is the expected payoff to the row player (maximizer) when both
/// players use their optimal mixed strategies.
#[derive(Clone, Debug)]
pub struct MatrixGameSolution {
    pub value: f64,
    pub row_strategy: Vec<f64>,
    pub col_strategy: Vec<f64>,
}

const EPS: f64 = 1e-9;

/// Solve a zero-sum matrix game exactly with the simplex method.
///
/// `payoff[i][j]` is the payoff to the row player when row plays i and
/// column plays j. Uses the classic reduction: shift the matrix so every
/// entry is positive, then solve
///     maximize sum(q)  subject to  A q <= 1,  q >= 0
/// The game value is 1 / sum(q) minus the shift. The column player's
/// strategy is q normalized; the row player's strategy is read from the
/// dual values (objective-row coefficients of the slack columns).
pub fn solve_matrix_game(payoff: &[Vec<f64>]) -> MatrixGameSolution {
    let m = payoff.len();
    let n = payoff[0].len();

    let min = payoff.iter().flatten().cloned().fold(f64::INFINITY, f64::min);
    let shift = 1.0 - min;

    // Tableau: m constraint rows + 1 objective row.
    // Columns: n game variables, m slack variables, 1 right-hand side.
    let width = n + m + 1;
    let rhs = width - 1;
    let mut t = vec![vec![0.0; width]; m + 1];
    for i in 0..m {
        for j in 0..n {
            t[i][j] = payoff[i][j] + shift;
        }
        t[i][n + i] = 1.0;
        t[i][rhs] = 1.0;
    }
    for j in 0..n {
        t[m][j] = -1.0;
    }
    let mut basis: Vec<usize> = (n..n + m).collect();

    // Bland's rule (smallest index) prevents cycling.
    while let Some(entering) = (0..rhs).find(|&j| t[m][j] < -EPS) {
        let mut leaving = None;
        let mut best_ratio = f64::INFINITY;
        for i in 0..m {
            if t[i][entering] > EPS {
                let ratio = t[i][rhs] / t[i][entering];
                if ratio < best_ratio - EPS {
                    best_ratio = ratio;
                    leaving = Some(i);
                }
            }
        }
        // Bounded: all shifted entries are positive.
        let leaving = leaving.expect("matrix game LP is bounded");

        let pivot = t[leaving][entering];
        for x in t[leaving].iter_mut() {
            *x /= pivot;
        }
        for i in 0..=m {
            if i != leaving {
                let factor = t[i][entering];
                if factor.abs() > EPS {
                    for j in 0..width {
                        t[i][j] -= factor * t[leaving][j];
                    }
                }
            }
        }
        basis[leaving] = entering;
    }

    let total = t[m][rhs];
    let mut col_strategy = vec![0.0; n];
    for (i, &b) in basis.iter().enumerate() {
        if b < n {
            col_strategy[b] = t[i][rhs] / total;
        }
    }
    let row_strategy = (0..m).map(|i| t[m][n + i] / total).collect();

    MatrixGameSolution {
        value: 1.0 / total - shift,
        row_strategy,
        col_strategy,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn assert_close(a: f64, b: f64) {
        assert!((a - b).abs() < 1e-6, "{} != {}", a, b);
    }

    #[test]
    fn matching_pennies_is_fifty_fifty_with_value_zero() {
        let s = solve_matrix_game(&[vec![1.0, -1.0], vec![-1.0, 1.0]]);
        assert_close(s.value, 0.0);
        assert_close(s.row_strategy[0], 0.5);
        assert_close(s.col_strategy[0], 0.5);
    }

    #[test]
    fn rock_paper_scissors_is_uniform() {
        let s = solve_matrix_game(&[
            vec![0.0, -1.0, 1.0],
            vec![1.0, 0.0, -1.0],
            vec![-1.0, 1.0, 0.0],
        ]);
        assert_close(s.value, 0.0);
        for p in s.row_strategy.iter().chain(s.col_strategy.iter()) {
            assert_close(*p, 1.0 / 3.0);
        }
    }

    #[test]
    fn saddle_point_gives_pure_strategies() {
        // Row 0 dominates; column 0 is best against it.
        let s = solve_matrix_game(&[vec![1.0, 2.0], vec![0.0, -1.0]]);
        assert_close(s.value, 1.0);
        assert_close(s.row_strategy[0], 1.0);
        assert_close(s.col_strategy[0], 1.0);
    }

    #[test]
    fn asymmetric_2x2_matches_closed_form() {
        // [[a, b], [c, d]] = [[2, -1], [-1, 1]]
        // value = (ad - bc) / (a + d - b - c) = 1/5, p(row 0) = (d - c) / 5 = 2/5
        let s = solve_matrix_game(&[vec![2.0, -1.0], vec![-1.0, 1.0]]);
        assert_close(s.value, 0.2);
        assert_close(s.row_strategy[0], 0.4);
        assert_close(s.col_strategy[0], 0.4);
    }

    #[test]
    fn one_turn_tug_of_war_blotto_example() {
        // docs/research-solution-methods.md: A has 3 players, B has 2, two ropes.
        // Pure maximin = 0, pure minimax = 1, mixed value = 0.5.
        let s = solve_matrix_game(&[
            vec![1.0, 0.0, 0.0],
            vec![1.0, 1.0, 0.0],
            vec![0.0, 1.0, 1.0],
            vec![0.0, 0.0, 1.0],
        ]);
        assert_close(s.value, 0.5);
    }

    #[test]
    fn non_square_game() {
        // Row player picks between two rows; column has three options.
        // Column 2 is dominated for the column player.
        let s = solve_matrix_game(&[vec![3.0, -1.0, 5.0], vec![-1.0, 1.0, 4.0]]);
        // Restricted to columns 0,1: [[3,-1],[-1,1]] -> value (3 - 1) / 6 = 1/3
        assert_close(s.value, 1.0 / 3.0);
        assert_close(s.col_strategy[2], 0.0);
        let row_sum: f64 = s.row_strategy.iter().sum();
        assert_close(row_sum, 1.0);
    }
}
