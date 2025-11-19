use leptos::*;
use crate::session::Session; 
// Note: You'll need to change the active_sim signal in App to hold a Session
use web_sys::{CanvasRenderingContext2d, HtmlCanvasElement};
use wasm_bindgen::JsCast;
use inference_engine::DiscoveryEvent;

#[component]
pub fn SimulationViewport(
    // Changed from Box<dyn Simulation> to the unified Session
    active_session: RwSignal<Option<Session>>,
    // We need a way to bubble up events to the feed
    on_discovery: Callback<DiscoveryEvent>, 
) -> impl IntoView {
    let canvas_ref = create_node_ref::<HtmlCanvasElement>();

    request_animation_frame_loop(move || {
        // 1. Update Logic
        if let Some(session) = active_session.get_untracked().as_mut() {
            // Run one tick of the Universe + Scientist
            if let Some(event) = session.tick() {
                on_discovery.call(event);
            }
            
            // 2. Render Logic
            if let Some(canvas) = canvas_ref.get_untracked() {
                 if let Ok(Some(ctx_val)) = canvas.get_context("2d") {
                    if let Ok(ctx) = ctx_val.dyn_into::<CanvasRenderingContext2d>() {
                        // (Assume draw_simulation is defined as before)
                        crate::components::simulation_viewport::draw_simulation(
                            &ctx, &canvas, session.get_state()
                        );
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
            style="border: 1px solid #444; background: black; display: block; margin: 2rem auto;"
        />
    }
}

// ... keep existing draw_simulation / draw_grid / draw_points functions ...
// ... draw functions (same as before) ...
fn draw_simulation(ctx: &CanvasRenderingContext2d, canvas: &HtmlCanvasElement, state: SimState) {
    let w = canvas.width() as f64;
    let h = canvas.height() as f64;
    ctx.set_fill_style(&"#000".into());
    ctx.fill_rect(0.0, 0.0, w, h);
    match state {
        SimState::Grid { width, height, cells, .. } => draw_grid(ctx, w, h, width, height, &cells),
        SimState::Points(points) => draw_points(ctx, w, h, &points),
    }
}
fn draw_grid(ctx: &CanvasRenderingContext2d, w: f64, h: f64, gw: u32, gh: u32, cells: &Vec<bool>) {
    if gw == 0 || gh == 0 { return; }
    let cw = w / gw as f64; let ch = h / gh as f64;
    ctx.set_fill_style(&"#00ff00".into());
    for (i, &alive) in cells.iter().enumerate() {
        if alive {
            let x = (i % gw as usize) as f64;
            let y = (i / gw as usize) as f64;
            ctx.fill_rect(x*cw, y*ch, cw.max(1.0), ch.max(1.0));
        }
    }
}
fn draw_points(ctx: &CanvasRenderingContext2d, w: f64, h: f64, points: &Vec<(f64, f64, f64)>) {
    if points.is_empty() { return; }
    let mut min_x = f64::INFINITY; let mut max_x = f64::NEG_INFINITY;
    let mut min_y = f64::INFINITY; let mut max_y = f64::NEG_INFINITY;
    let relevant = if points.len() > 5000 { &points[points.len()-5000..] } else { points };
    for (x, y, _) in relevant { min_x = min_x.min(*x); max_x = max_x.max(*x); min_y = min_y.min(*y); max_y = max_y.max(*y); }
    let sx = (max_x - min_x).max(1e-6) * 1.2; let sy = (max_y - min_y).max(1e-6) * 1.2;
    min_x -= sx*0.1; min_y -= sy*0.1;
    ctx.set_fill_style(&"#00aaff".into());
    for (x, y, _) in relevant {
        ctx.fill_rect(((x - min_x)/sx)*w, ((y - min_y)/sy)*h, 1.5, 1.5);
    }
}
