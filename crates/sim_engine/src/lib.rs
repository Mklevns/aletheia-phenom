//! The "Simulation Engine" Crate Root

use serde::{Deserialize, Serialize};
pub use ready::{CellPattern, MacroCell}; 

// --- Module Registration ---
pub mod gol;
pub mod ode;
pub mod gray_scott; // <--- DON'T FORGET THIS LINE (Registers the new file)

// --- Shared Trait ---
pub trait Simulation {
    fn new() -> Self where Self: Sized;

    fn step(&mut self);

    fn get_state(&self) -> SimState;

    fn set_param(&mut self, key: &str, value: ParamValue);
    
    fn as_experimentable(&mut self) -> Option<&mut dyn Experimentable> {
        None
    } // <--- Closing brace for function
} // <--- Closing brace for trait

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
    
    // NEW: For Reaction-Diffusion (0.0 to 1.0 intensity map)
    FloatGrid {
        width: u32,
        height: u32,
        values: Vec<f64>,
    }
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

#[derive(Clone, Debug)]
pub enum Action {
    FlipCell { r: usize, c: usize },
    Perturb { which: u8, delta: f64 },
    SetParam { name: String, value: f64 },
    Noop,
}

#[derive(Clone, Debug)]
pub enum Observation {
    GridSummary { alive: usize, width: usize, height: usize },
    StateVec([f64; 3]),
    Text(String),
    None,
}

pub trait Experimentable {
    fn apply_action(&mut self, action: Action);
    fn observe(&self) -> Observation;
    fn reward(&self) -> f64;
}
