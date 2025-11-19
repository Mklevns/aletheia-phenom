use super::{ParamValue, SimState, Simulation, Experimentable, Action, Observation};
use serde::Serialize;
use std::f64::consts::PI;

#[derive(Clone)]
pub struct GrayScott {
    width: usize,
    height: usize,
    // Flattened grids for cache locality
    u: Vec<f64>,
    v: Vec<f64>,
    // Double buffering
    next_u: Vec<f64>,
    next_v: Vec<f64>,
    
    // Physics Parameters
    f: f64, // Feed rate
    k: f64, // Kill rate
    da: f64, // Diffusion A (U)
    db: f64, // Diffusion B (V)
    dt: f64, // Time step
}

impl GrayScott {
    // Laplacian convolution weights (3x3 isotropic)
    // Center: -1.0
    // Adjacent: 0.2
    // Diagonal: 0.05
    // Sum = 0.0
    
    pub fn init(width: usize, height: usize) -> Self {
        let size = width * height;
        let mut sim = Self {
            width,
            height,
            u: vec![1.0; size], // U starts full everywhere
            v: vec![0.0; size], // V starts empty
            next_u: vec![1.0; size],
            next_v: vec![0.0; size],
            f: 0.055, // "Coral" preset
            k: 0.062,
            da: 1.0,
            db: 0.5,
            dt: 1.0,
        };
        sim.seed_center();
        sim
    }

    fn seed_center(&mut self) {
        let cx = self.width / 2;
        let cy = self.height / 2;
        let r = 10;
        for y in (cy - r)..(cy + r) {
            for x in (cx - r)..(cx + r) {
                let idx = y * self.width + x;
                if idx < self.u.len() {
                    self.v[idx] = 1.0; // Inject V
                }
            }
        }
    }
    
    // Helper for toroidal wrapping (wrapping around edges)
    #[inline(always)]
    fn idx(&self, x: isize, y: isize) -> usize {
        let w = self.width as isize;
        let h = self.height as isize;
        let wrapped_x = (x + w) % w;
        let wrapped_y = (y + h) % h;
        (wrapped_y * w + wrapped_x) as usize
    }
}

impl Simulation for GrayScott {
    fn new() -> Self {
        Self::init(128, 128) // 128x128 is good balance for WASM
    }

    fn step(&mut self) {
        let w = self.width as isize;
        let h = self.height as isize;

        // OPTIMIZATION: We explicitly iterate indices to avoid bounds checking overhead 
        // in a tight loop if possible, but for safety + wrapping we use helper.
        // (For maximum speed in Rust, we would use unsafe pointers or 1D iterators with windowing, 
        // but this index math is sufficient for < 500x500 grids).
        for y in 0..h {
            for x in 0..w {
                let i = (y * w + x) as usize;
                
                let u = self.u[i];
                let v = self.v[i];
                
                // Calculate Laplacian
                let mut lap_u = 0.0;
                let mut lap_v = 0.0;
                
                // 9-point stencil
                let neighbors = [
                    (-1,-1, 0.05), (0,-1, 0.2), (1,-1, 0.05),
                    (-1, 0, 0.2),  (0, 0, -1.0), (1, 0, 0.2),
                    (-1, 1, 0.05), (0, 1, 0.2), (1, 1, 0.05)
                ];

                for &(dx, dy, weight) in &neighbors {
                    let ni = self.idx(x + dx, y + dy);
                    lap_u += self.u[ni] * weight;
                    lap_v += self.v[ni] * weight;
                }

                let uvv = u * v * v;
                
                // Gray-Scott Reaction Equations
                let du = (self.da * lap_u - uvv + self.f * (1.0 - u)) * self.dt;
                let dv = (self.db * lap_v + uvv - (self.f + self.k) * v) * self.dt;

                self.next_u[i] = (u + du).clamp(0.0, 1.0);
                self.next_v[i] = (v + dv).clamp(0.0, 1.0);
            }
        }
        
        std::mem::swap(&mut self.u, &mut self.next_u);
        std::mem::swap(&mut self.v, &mut self.next_v);
    }

    fn get_state(&self) -> SimState {
        // We map the 'V' chemical to a 0.0-1.0 intensity grid for rendering
        SimState::FloatGrid {
            width: self.width as u32,
            height: self.height as u32,
            values: self.v.clone(),
        }
    }

    fn set_param(&mut self, key: &str, value: ParamValue) {
        if let ParamValue::Float(v) = value {
            match key {
                "f" => self.f = v,
                "k" => self.k = v,
                _ => {}
            }
        }
    }

    fn as_experimentable(&mut self) -> Option<&mut dyn Experimentable> {
        Some(self)
    }
}

impl Experimentable for GrayScott {
    fn apply_action(&mut self, action: Action) {
        match action {
            Action::Perturb { which, delta } => {
                // If the agent kicks "which=0", we add V at a random spot
                // (Modeling local chemical injection)
                if which == 0 {
                    // HACK: Use delta as seed for position to be deterministic-ish
                    let cx = (self.width as f64 * (delta.abs() % 1.0)) as usize;
                    let cy = (self.height as f64 * ((delta * 10.0).abs() % 1.0)) as usize;
                    let r = 4;
                    for y in 0..self.height {
                        for x in 0..self.width {
                           let dx = x as isize - cx as isize;
                           let dy = y as isize - cy as isize;
                           if dx*dx + dy*dy < r*r {
                               let idx = y * self.width + x;
                               self.v[idx] = (self.v[idx] + 0.5).min(1.0);
                           }
                        }
                    }
                }
            },
            Action::SetParam { name, value } => {
                if name == "f" { self.f = value; }
                if name == "k" { self.k = value; }
            }
            _ => {}
        }
    }

    fn observe(&self) -> Observation {
        // The agent sees the "Total Mass" of V and the "Entropy" (rough approx)
        let total_v: f64 = self.v.iter().sum();
        Observation::StateVec([total_v, self.f, self.k])
    }

    fn reward(&self) -> f64 {
        // Reward: Keep the reaction ALIVE.
        // If V dies out (0) -> Reward 0.
        // If V takes over everything (screen fills) -> Reward 0 (Saturation).
        // Ideal: ~20% coverage (complex patterns).
        let coverage = self.v.iter().sum::<f64>() / (self.width * self.height) as f64;
        
        // Gaussian curve peaked at 0.2
        (-((coverage - 0.2).powi(2)) * 100.0).exp() * 10.0
    }
}
