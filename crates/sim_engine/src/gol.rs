//! High-performance Conway's Game of Life using macroquad-compatible HashLife via `ready`

use super::{ParamValue, SimState, Simulation, Experimentable, Action, Observation};
use ready::{
    CellPattern, HashLife, MacroCell, Node, Pattern, PatternID, Universe, UniverseExt,
};
use std::collections::HashMap;

pub struct GameOfLife {
    universe: Universe,
    generation: u64,
    view_width_cells: u32,
    view_height_cells: u32,
    view_offset_x: i64,
    view_offset_y: i64,
    pattern_library: HashMap<String, PatternID>,
}

impl GameOfLife {
    fn default_pattern() -> PatternID {
        let r_pentomino = CellPattern::from_rle(
            "b2o$2o$bo!", 
        ).unwrap();
        r_pentomino.id()
    }
}

impl Simulation for GameOfLife {
    fn new() -> Self {
        let mut universe = Universe::new();
        let root = Self::default_pattern();
        universe.set_root(root);
        
        let mut pattern_library = HashMap::new();
        pattern_library.insert("glider".into(), CellPattern::glider().id());

        Self {
            universe,
            generation: 0,
            view_width_cells: 256,
            view_height_cells: 256,
            view_offset_x: -128,
            view_offset_y: -128,
            pattern_library,
        }
    }

    fn step(&mut self) {
        self.universe.step();
        self.generation += 1;
    }

    fn get_state(&self) -> SimState {
        let macro_width = self.view_width_cells.next_power_of_two() as i64;
        let macro_height = self.view_height_cells.next_power_of_two() as i64;

        let bitmap = self.universe.render(
            self.view_offset_x,
            self.view_offset_y,
            macro_width,
            macro_height,
        );

        let cells: Vec<bool> = bitmap
            .iter()
            .flatten()
            .map(|&cell| cell == MacroCell::Alive)
            .collect();

        SimState::Grid {
            offset_x: self.view_offset_x,
            offset_y: self.view_offset_y,
            width: self.view_width_cells,
            height: self.view_height_cells,
            cells,
        }
    }

    fn set_param(&mut self, key: &str, value: ParamValue) {
        match (key, value) {
            ("inject_pattern", ParamValue::String(name)) => {
                if let Some(&pattern_id) = self.pattern_library.get(&name) {
                    let centered = pattern_id.at(
                        self.view_offset_x + self.view_width_cells as i64 / 2,
                        self.view_offset_y + self.view_height_cells as i64 / 2,
                    );
                    self.universe.set_root(centered);
                }
            }
            _ => {}
        }
    }
    
    // Hook up the interface
    fn as_experimentable(&mut self) -> Option<&mut dyn Experimentable> {
        Some(self)
    }
}

// --- RL Implementation ---
impl Experimentable for GameOfLife {
    fn apply_action(&mut self, action: Action) {
        match action {
            Action::FlipCell { r, c } => {
                // Map viewport r,c to world coordinates
                let world_x = self.view_offset_x + c as i64;
                let world_y = self.view_offset_y + r as i64;
                self.universe.set_cell(world_x, world_y, true); // For simplicity, we just birth cells
            }
            _ => {}
        }
    }

    fn observe(&self) -> Observation {
        // Return alive count estimate (very rough for hashlife, but usable)
        // For now, just use view dimensions as dummy
        Observation::GridSummary {
            alive: 0, // Hashlife counting is expensive, assume 0 for mock
            width: self.view_width_cells as usize,
            height: self.view_height_cells as usize,
        }
    }

    fn reward(&self) -> f64 {
        // Simple reward: generation count (survival)
        self.generation as f64
    }
}

impl Default for GameOfLife {
    fn default() -> Self {
        Self::new()
    }
}
