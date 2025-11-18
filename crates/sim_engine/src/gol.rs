//! High-performance Conway's Game of Life using macroquad-compatible HashLife via `ready`
//! Implements the Simulation trait and returns state in a frontend-friendly format.

use super::{ParamValue, SimState, Simulation}; // <-- IMPORT from lib.rs
use ready::{
    CellPattern, HashLife, MacroCell, Node, Pattern, PatternID, Universe, UniverseExt,
};
use std::collections::HashMap;

/// Conway's Game of Life implementation using ready::HashLife
pub struct GameOfLife {
    universe: Universe,
    /// Current generation (logical, not internal HashLife step count)
    generation: u64,
    /// Viewport definition
    view_width_cells: u32,
    view_height_cells: u32,
    view_offset_x: i64,
    view_offset_y: i64,
    /// Cache of pre-loaded famous patterns for quick injection
    pattern_library: HashMap<String, PatternID>,
}

impl GameOfLife {
    /// Create a new infinite HashLife universe with a reasonable default viewport
    /// Starts with the classic R-pentomino in the center for immediate action
    fn default_pattern() -> PatternID {
        let r_pentomino = CellPattern::from_rle(
            "b2o$2o$bo!", // A simple R-pentomino
        )
        .unwrap();
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
        pattern_library.insert("lwss".into(), CellPattern::lwss().id());
        pattern_library.insert("gosper_glider_gun".into(), CellPattern::gosper_glider_gun().id());
        pattern_library.insert("block".into(), CellPattern::block().id());
        pattern_library.insert("beehive".into(), CellPattern::beehive().id());
        pattern_library.insert("blinker".into(), CellPattern::blinker().id());
        // Add Grok's RLE
        pattern_library.insert("grok_default".into(), CellPattern::from_rle("23bo$22bobo$12b2o6b2o12b2o$11bo3bo4b2o12b2o$2o8bo5bo3b2o$2o8bo3bob2o4bo$10bo5bo7b2o$11bo3bo9b2o$12b2o!").unwrap().id());


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
        // HashLife will internally jump arbitrarily far ahead when beneficial
        self.universe.step();
        self.generation += 1;
    }

    fn get_state(&self) -> SimState {
        // Determine the macro-cell size we need for this viewport
        let cell_size = self
            .universe
            .root()
            .map(|n| n.level())
            .unwrap_or(0)
            .max(4); // never go below 16×16 macrocells

        let macro_width = self.view_width_cells.next_power_of_two() as i64;
        let macro_height = self.view_height_cells.next_power_of_two() as i64;

        // Extract a dense bitmap from the HashLife tree
        let bitmap = self.universe.render(
            self.view_offset_x,
            self.view_offset_y,
            macro_width,
            macro_height,
        );

        // Flatten into row-major bool vector
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
            ("inject_rle", ParamValue::String(rle)) => {
                if let Ok(pattern) = CellPattern::from_rle(&rle) {
                    let centered = pattern.at(
                        self.view_offset_x + self.view_width_cells as i64 / 2,
                        self.view_offset_y + self.view_height_cells as i64 / 2,
                    );
                    self.universe.set_root(centered);
                }
            }
            ("viewport_size", ParamValue::Int(size)) => {
                let s = size.max(64).next_power_of_two() as u32;
                self.view_width_cells = s;
                self.view_height_cells = s;
                self.view_offset_x = -(s as i64) / 2;
                self.view_offset_y = -(s as i64) / 2;
            }
            ("pan", ParamValue::Int(dx)) => {
                self.view_offset_x += dx;
                self.view_offset_y += dx; // simple diagonal pan for now
            }
            _ => { /* ignore unknown params gracefully */ }
        }
    }
}

impl Default for GameOfLife {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn creates_and_steps() {
        let mut gol = GameOfLife::new();
        let state0 = gol.get_state();
        gol.step();
        let state1 = gol.get_state();
        assert_ne!(state0, state1);
    }
}
