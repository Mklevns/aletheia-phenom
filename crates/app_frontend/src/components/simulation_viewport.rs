use leptos::*;
use sim_engine::{SimState, Simulation};
use wasm_bindgen::JsCast;
use web_sys::{CanvasRenderingContext2d, HtmlCanvasElement};

/// Props:
///   active_sim: The global simulation signal
#[component]
pub fn SimulationViewport(
    active_sim: RwSignal<Option<Box<dyn Simulation>>>,
) -> impl IntoView {
    // Canvas node reference (persistent across re-renders).
    let canvas_ref = create_node_ref::<HtmlCanvasElement>();

    // The heartbeat: a 60fps loop using requestAnimationFrame.
    request_animation_frame_loop(move || {
        // We only step and draw if a simulation is loaded
        if active_sim.get_untracked().is_some() {
            // 1. Step the simulation
            // We use .update() to mutate the simulation in-place
            active_sim.update(|sim_opt| {
                if let Some(sim) = sim_opt.as_mut() {
                    sim.step();
                }
            });

            // 2. Draw the current state
            // We use .get_untracked() for drawing to avoid circular dependencies
            if let Some(canvas) = canvas_ref.get_untracked() {
                if let Some(sim) = active_sim.get_untracked() {
                    if let Ok(Some(ctx_val)) = canvas.get_context("2d") {
                        if let Ok(ctx) = ctx_val.dyn_into::<CanvasRenderingContext2d>() {
                            draw_simulation(&ctx, &canvas, sim.get_state());
                        }
                    }
                }
            }
        } else {
            // No sim loaded, clear the canvas
            if let Some(canvas) = canvas_ref.get_untracked() {
                 if let Ok(Some(ctx_val)) = canvas.get_context("2d") {
                        if let Ok(ctx) = ctx_val.dyn_into::<CanvasRenderingContext2d>() {
                            let w = canvas.width() as f64;
                            let h = canvas.height() as f64;
                            ctx.set_fill_style(&"#000".into());
                            ctx.fill_rect(0.0, 0.0, w, h);
                        }
                    }
            }
        }
    });

    view! {
        <canvas
            node_ref=canvas_ref
            width="800"
            height="600"
            style="
                border: 1px solid #444;
                background: black;
                display: block;
                margin: 2rem auto;
                width: 80%;
                max-width: 800px;
            "
        />
    }
}

/// Draw the simulation state onto the canvas.
fn draw_simulation(
    ctx: &CanvasRenderingContext2d,
    canvas: &HtmlCanvasElement,
    state: SimState,
) {
    let w = canvas.width() as f64;
    let h = canvas.height() as f64;

    // Clear the frame
    ctx.set_fill_style(&"#000".into());
    ctx.fill_rect(0.0, 0.0, w, h);

    match state {
        SimState::Grid {
            width,
            height,
            cells,
            ..
        } => {
            draw_grid(ctx, w, h, width, height, &cells);
        }

        SimState::Points(points) => {
            draw_points(ctx, w, h, &points);
        }
    }
}

/// --- REFACTORED FUNCTION ---
/// Draw a 2D Cellular Automaton (Game of Life) from a FLAT Vec<bool>
fn draw_grid(
    ctx: &CanvasRenderingContext2d,
    canvas_width: f64,
    canvas_height: f64,
    grid_width: u32,
    grid_height: u32,
    cells: &Vec<bool>,
) {
    if grid_width == 0 || grid_height == 0 || cells.is_empty() {
        return;
    }

    let cell_w = canvas_width / grid_width as f64;
    let cell_h = canvas_height / grid_height as f64;

    ctx.set_fill_style(&"#00ff00".into()); // Green live cells

    for (i, &is_alive) in cells.iter().enumerate() {
        if is_alive {
            let x = (i % grid_width as usize) as f64;
            let y = (i / grid_width as usize) as f64;

            ctx.fill_rect(
                x * cell_w,
                y * cell_h,
                cell_w.max(1.0), // Ensure at least 1px
                cell_h.max(1.0), // Ensure at least 1px
            );
        }
    }
}

/// Draw Lorenz/Rössler attractor tail (Grok's code, unchanged, it's perfect)
fn draw_points(
    ctx: &CanvasRenderingContext2d,
    w: f64,
    h: f64,
    points: &Vec<(f64, f64, f64)>,
) {
    if points.is_empty() {
        return;
    }

    // Compute bounds
    let mut min_x = f64::INFINITY;
    let mut max_x = f64::NEG_INFINITY;
    let mut min_y = f64::INFINITY;
    let mut max_y = f64::NEG_INFINITY;

    // Use a subset for performance if tail is huge
    let relevant_points = if points.len() > 5000 {
        &points[points.len() - 5000..]
    } else {
        points
    };

    for (x, y, _z) in relevant_points.iter() {
        min_x = min_x.min(*x);
        max_x = max_x.max(*x);
        min_y = min_y.min(*y);
        max_y = max_y.max(*y);
    }
    
    // Add padding to bounds
    let span_x = (max_x - min_x).max(1e-6);
    let span_y = (max_y - min_y).max(1e-6);
    min_x -= span_x * 0.1;
    min_y -= span_y * 0.1;
    let span_x = span_x * 1.2;
    let span_y = span_y * 1.2;


    // Draw points as glowing cyan
    ctx.set_fill_style(&"#00aaff".into());

    for (x, y, _z) in relevant_points {
        let px = ((x - min_x) / span_x) * w;
        let py = ((y - min_y) / span_y) * h;

        // Draw tiny rectangle as a point
        ctx.fill_rect(px, py, 1.5, 1.5);
    }
}
