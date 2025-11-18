//! Dynamical Systems (ODE) Module
//!
//! Implements the Simulation trait for continuous chaotic attractors
//! like Lorenz and Rössler.

use super::{ParamValue, SimState, Simulation}; // <-- IMPORT from lib.rs
use diffeq_rs::prelude::*;
use serde::Serialize;

/// Maximum number of historical points to keep.
/// This is ideal for rendering the attractor tail.
const MAX_HISTORY: usize = 800;

/// Supported dynamical systems.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum ODESystem {
    Lorenz,
    Rossler,
}

/// Internal state for ODE simulation.
pub struct ODESim {
    pub system: ODESystem,
    pub params: ODEParams,
    pub state: [f64; 3],
    pub dt: f64,
    pub tail: Vec<(f64, f64, f64)>,
}

/// Parameters for Lorenz or Rössler systems.
#[derive(Debug, Clone, Copy)]
pub struct ODEParams {
    // Lorenz
    pub sigma: f64,
    pub rho: f64,
    pub beta: f64,
    // Rössler
    pub a: f64,
    pub b: f64,
    pub c: f64,
}

impl Default for ODEParams {
    fn default() -> Self {
        Self {
            // Lorenz default
            sigma: 10.0,
            rho: 28.0,
            beta: 8.0 / 3.0,
            // Rossler default
            a: 0.2,
            b: 0.2,
            c: 5.7,
        }
    }
}

impl Simulation for ODESim {
    fn new() -> Self {
        Self {
            system: ODESystem::Lorenz,
            params: ODEParams::default(),
            state: [1.0, 1.0, 1.0],
            dt: 0.01,
            tail: Vec::with_capacity(MAX_HISTORY),
        }
    }

    /// Advance the ODE one time-step.
    fn step(&mut self) {
        let dt = self.dt;

        // Use diffeq-rs RK4 integrator
        let next = rk4_step(self.state, dt, |state| self.deriv(state));

        self.state = next;

        // Add to the rolling tail buffer.
        self.tail.push((next[0], next[1], next[2]));
        if self.tail.len() > MAX_HISTORY {
            self.tail.drain(0..(self.tail.len() - MAX_HISTORY));
        }
    }

    /// Returns SimState::Points with the recent tail of the attractor.
    fn get_state(&self) -> SimState {
        SimState::Points(self.tail.clone())
    }

    /// Dynamically set parameters or switch systems using the ParamValue enum.
    fn set_param(&mut self, name: &str, value: ParamValue) {
        match name {
            // Lorenz params
            "sigma" => {
                if let ParamValue::Float(v) = value {
                    self.params.sigma = v
                }
            }
            "rho" => {
                if let ParamValue::Float(v) = value {
                    self.params.rho = v
                }
            }
            "beta" => {
                if let ParamValue::Float(v) = value {
                    self.params.beta = v
                }
            }

            // Rossler params
            "a" => {
                if let ParamValue::Float(v) = value {
                    self.params.a = v
                }
            }
            "b" => {
                if let ParamValue::Float(v) = value {
                    self.params.b = v
                }
            }
            "c" => {
                if let ParamValue::Float(v) = value {
                    self.params.c = v
                }
            }

            // Change system
            "system" => {
                if let ParamValue::String(s) = value {
                    match s.as_str() {
                        "lorenz" => self.system = ODESystem::Lorenz,
                        "rossler" => self.system = ODESystem::Rossler,
                        _ => {}
                    }
                    self.reset_state();
                }
            }

            // Reset state vector
            "reset" => {
                if let ParamValue::Bool(true) = value {
                    self.reset_state();
                }
            }
            _ => {
                // Unknown parameter — ignore gracefully.
            }
        }
    }
}

impl ODESim {
    /// Reset initial condition & clear tail.
    fn reset_state(&mut self) {
        self.state = [1.0, 1.0, 1.0];
        self.tail.clear();
    }

    /// Compute derivative for the chosen system.
    fn deriv(&self, s: [f64; 3]) -> [f64; 3] {
        match self.system {
            ODESystem::Lorenz => self.lorenz(s),
            ODESystem::Rossler => self.rossler(s),
        }
    }

    // --- Private helper functions ---

    /// Lorenz attractor
    fn lorenz(&self, s: [f64; 3]) -> [f64; 3] {
        let (x, y, z) = (s[0], s[1], s[2]);
        let p = self.params;
        [
            p.sigma * (y - x),
            x * (p.rho - z) - y,
            x * y - p.beta * z,
        ]
    }

    /// Rössler attractor
    fn rossler(&self, s: [f64; 3]) -> [f64; 3] {
        let (x, y, z) = (s[0], s[1], s[2]);
        let p = self.params;
        [
            -y - z,
            x + p.a * y,
            p.b + z * (x - p.c),
        ]
    }
}

// --- Grok's no-allocation RK4 implementation ---

/// Convenience wrapper so code can say `rk4_step(initial, dt, f)`
fn rk4_step<const N: usize, F>(state: [f64; N], dt: f64, f: F) -> [f64; N]
where
    F: Fn([f64; N]) -> [f64; N],
{
    let k1 = f(state);
    let k2 = f(add_arrays(state, mul_array(k1, dt * 0.5)));
    let k3 = f(add_arrays(state, mul_array(k2, dt * 0.5)));
    let k4 = f(add_arrays(state, mul_array(k3, dt)));

    add_arrays(
        state,
        mul_array(
            add_arrays(
                add_arrays(k1, mul_array(k2, 2.0)),
                add_arrays(mul_array(k3, 2.0), k4),
            ),
            dt / 6.0,
        ),
    )
}

/// Helper vector math to avoid allocations.
fn add_arrays<const N: usize>(a: [f64; N], b: [f64; N]) -> [f64; N] {
    let mut out = [0.0; N];
    for i in 0..N {
        out[i] = a[i] + b[i];
    }
    out
}

fn mul_array<const N: usize>(a: [f64; N], k: f64) -> [f64; N] {
    let mut out = [0.0; N];
    for i in 0..N {
        out[i] = a[i] * k;
    }
    out
}
