//! The "Simulation Engine" Crate Root
//!
//! This file defines the shared traits and data structures that all
//! simulations in Aletheia-Phenom must implement.

use serde::{Deserialize, Serialize};
pub use ready::{CellPattern, MacroCell}; // Re-export for frontend

// --- Module Registration ---
// This line makes the code in "gol.rs" available as `sim_engine::gol`
pub mod gol;

// --- Shared Trait ---
/// The canonical Simulation trait (as defined in lib.rs)
pub trait Simulation {
    /// Create a new simulation instance
    fn new() -> Self
    where
        Self: Sized;

    /// Advance the simulation by one logical generation (may compute far ahead with HashLife)
    fn step(&mut self);

    /// Get the current visible state in a format the frontend can render efficiently
    fn get_state(&self) -> SimState;

    /// Optional: set runtime parameters (e.g. inject patterns, resize viewport, etc.)
    fn set_param(&mut self, key: &str, value: ParamValue);
}

// --- Shared Data Structures ---

/// Unified state representation for all simulations in Aletheia-Phenom
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub enum SimState {
    Grid {
        /// Top-left corner of the visible viewport (in cell coordinates)
        offset_x: i64,
        offset_y: i64,
        /// Width and height in cells (power-of-two recommended for HashLife)
        width: u32,
        height: u32,
        /// Flattened row-major vec of alive (true) / dead (false)
        cells: Vec<bool>,
    },
    // Future variants: Graph(...), PhaseSpace(...), etc.
}

/// Parameter values that can be injected at runtime
#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum ParamValue {
    Bool(bool),
    Int(i64),
    Float(f64),
    String(String),
    Pattern(CellPattern),
}
