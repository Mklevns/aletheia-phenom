use super::{ParamValue, SimState, Simulation, Experimentable, Action, Observation};
use diffeq_rs::prelude::*;
use serde::Serialize;

const MAX_HISTORY: usize = 800;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum ODESystem { Lorenz, Rossler }

pub struct ODESim {
    pub system: ODESystem,
    pub params: ODEParams,
    pub state: [f64; 3],
    pub dt: f64,
    pub tail: Vec<(f64, f64, f64)>,
}

#[derive(Debug, Clone, Copy)]
pub struct ODEParams {
    pub sigma: f64, pub rho: f64, pub beta: f64,
    pub a: f64, pub b: f64, pub c: f64,
}

impl Default for ODEParams {
    fn default() -> Self {
        Self {
            sigma: 10.0, rho: 28.0, beta: 8.0 / 3.0,
            a: 0.2, b: 0.2, c: 5.7,
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

    fn step(&mut self) {
        let next = rk4_step(self.state, self.dt, |state| self.deriv(state));
        self.state = next;
        self.tail.push((next[0], next[1], next[2]));
        if self.tail.len() > MAX_HISTORY {
            self.tail.drain(0..(self.tail.len() - MAX_HISTORY));
        }
    }

    fn get_state(&self) -> SimState {
        SimState::Points(self.tail.clone())
    }

    fn set_param(&mut self, _name: &str, _value: ParamValue) {
        // Standard UI parameter setting (optional stub)
    }
    
    fn as_experimentable(&mut self) -> Option<&mut dyn Experimentable> {
        Some(self)
    }
}

impl ODESim {
    fn reset_state(&mut self) { self.state = [1.0, 1.0, 1.0]; self.tail.clear(); }
    fn deriv(&self, s: [f64; 3]) -> [f64; 3] {
        match self.system {
            ODESystem::Lorenz => self.lorenz(s),
            ODESystem::Rossler => self.rossler(s),
        }
    }
    fn lorenz(&self, s: [f64; 3]) -> [f64; 3] {
        let (x, y, z) = (s[0], s[1], s[2]);
        let p = self.params;
        [p.sigma * (y - x), x * (p.rho - z) - y, x * y - p.beta * z]
    }
    fn rossler(&self, s: [f64; 3]) -> [f64; 3] {
        let (x, y, z) = (s[0], s[1], s[2]);
        let p = self.params;
        [-y - z, x + p.a * y, p.b + z * (x - p.c)]
    }
}

// --- RL Implementation ---
impl Experimentable for ODESim {
    // MERGED ACTION HANDLER
    fn apply_action(&mut self, action: Action) {
        match action {
            Action::Perturb { which, delta } => {
                if which < 3 { self.state[which as usize] += delta; }
            }
            // --- NEW: Allow AI to tune constants ---
            Action::SetParam { name, value } => {
                match name.as_str() {
                    "sigma" => self.params.sigma = value,
                    "rho"   => self.params.rho   = value,
                    "beta"  => self.params.beta  = value,
                    _ => {}
                }
            }
            _ => {}
        }
    }

    fn observe(&self) -> Observation {
        Observation::StateVec(self.state)
    }

    fn reward(&self) -> f64 {
        // Reward distance from origin (energy)
        (self.state[0].powi(2) + self.state[1].powi(2) + self.state[2].powi(2)).sqrt()
    }
}

// ... rk4 helper ...
fn rk4_step<const N: usize, F>(state: [f64; N], dt: f64, f: F) -> [f64; N]
where F: Fn([f64; N]) -> [f64; N] {
    let k1 = f(state);
    let k2 = f(add_arrays(state, mul_array(k1, dt * 0.5)));
    let k3 = f(add_arrays(state, mul_array(k2, dt * 0.5)));
    let k4 = f(add_arrays(state, mul_array(k3, dt)));
    add_arrays(state, mul_array(add_arrays(add_arrays(k1, mul_array(k2, 2.0)), add_arrays(mul_array(k3, 2.0), k4)), dt / 6.0))
}
fn add_arrays<const N: usize>(a: [f64; N], b: [f64; N]) -> [f64; N] {
    let mut out = [0.0; N]; for i in 0..N { out[i] = a[i] + b[i]; } out
}
fn mul_array<const N: usize>(a: [f64; N], k: f64) -> [f64; N] {
    let mut out = [0.0; N]; for i in 0..N { out[i] = a[i] * k; } out
}
