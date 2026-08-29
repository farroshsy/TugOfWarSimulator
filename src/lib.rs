pub mod game;
pub mod strategies;
pub mod algorithms;
pub mod experiments;
pub mod ui;

pub use strategies::Strategy;
pub use strategies::Allocation;
pub use algorithms::{DpSolver, DpStrategy, DpFeasibilityEstimator};