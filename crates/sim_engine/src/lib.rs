//! The "Simulation Engine" Crate Root
//!
//! This file defines the shared traits and data structures that all
//! simulations in Aletheia-Phenom must implement.

use serde::{Deserialize, Serialize};
pub use ready::{CellPattern, MacroCell}; // Re-export for frontend

// --- Module Registration ---
pub mod gol;
pub mod ode;

// --- Shared Trait ---
/// The canonical Simulation trait
pub trait Simulation {
    /// Create a new simulation instance
    fn new() -> Self
    where
        Self: Sized;

    /// Advance the simulation by one logical generation
    fn step(&mut self);

    /// Get the current visible state in a format the frontend can render efficiently
    fn get_state(&self) -> SimState;

    /// Set runtime parameters
    fn set_param(&mut self, key: &str, value: ParamValue);
    
    /// Optional: Get a reference to the experimentable interface if supported
    fn as_experimentable(&mut self) -> Option<&mut dyn Experimentable> {
        None
    }
}

// --- Shared Data Structures ---

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub enum SimState {
    Grid {
        offset_x: i64,
        offset_y: i64,
        width: u32,
        height: u32,
        cells: Vec<bool>,
    },
    Points(Vec<(f64, f64, f64)>),
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum ParamValue {
    Bool(bool),
    Int(i64),
    Float(f64),
    String(String),
    Pattern(CellPattern),
}

// --- EXPERIMENTAL INTERFACE (RL / Agent Hooks) ---

/// An action the experimenter can apply to a simulation.
#[derive(Clone, Debug)]
pub enum Action {
    /// For CA: flip cell at (r,c) relative to viewport
    FlipCell { r: usize, c: usize },
    /// For ODE: add small delta to variable x|y|z
    Perturb { which: u8, delta: f64 },
    /// Change parameter (by name)
    SetParam { name: String, value: f64 },
    /// No-op / wait
    Noop,
}

/// Lightweight observation returned by simulation to feed the agent.
#[derive(Clone, Debug)]
pub enum Observation {
    /// CA: summary stats
    GridSummary { alive: usize, width: usize, height: usize },
    /// ODE: latest state vector
    StateVec([f64; 3]),
    /// Generic textual debug
    Text(String),
    /// Empty/None
    None,
}

/// Optional extension trait: a simulation that can be directly experimented with.
pub trait Experimentable {
    /// Apply an action (mutates simulation).
    fn apply_action(&mut self, action: Action);

    /// Return a compact observation suitable for the agent.
    fn observe(&self) -> Observation;

    /// Compute scalar reward for the last step (agent-specific).
    fn reward(&self) -> f64;
}
