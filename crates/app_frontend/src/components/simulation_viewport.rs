use leptos::*;
use crate::session::Session;
use sim_engine::SimState;
use inference_engine::DiscoveryEvent;
use wasm_bindgen::JsCast;
use web_sys::{CanvasRenderingContext2d, HtmlCanvasElement};

#[component]
pub fn SimulationViewport(
    active_session: RwSignal<Option<Session>>,
    is_playing: ReadSignal<bool>,
    speed: ReadSignal<f64>,
    step_trigger: ReadSignal<i32>,     
    set_tick_count: WriteSignal<u64>,  
    #[prop(into)]
    on_discovery: Callback<DiscoveryEvent>,
) -> impl IntoView {
    let canvas_ref = create_node_ref::<HtmlCanvasElement>();
    
    // Timing state
    let last_frame_time = create_rw_signal(0.0);
    let accumulator = create_rw_signal(0.0);
    let last_step_trigger = create_rw_signal(0);

    request_animation_frame_loop(move || {
        // Calculate Delta Time
        let now = js_sys::Date::now();
        let prev = last_frame_time.get_untracked();
        last_frame_time.set(now);
        
        // Init timestamp on first run
        if prev == 0.0 { return; }
        
        let dt_ms = now - prev;
        // Cap dt to prevent spiral of death if tab is backgrounded
        let safe_dt = if dt_ms > 100.0 { 100.0 } else { dt_ms };

        if let Some(session) = active_session.get_untracked().as_mut() {
            
            // 1. Handle Manual Step (Click)
            let current_trigger = step_trigger.get();
            if current_trigger > last_step_trigger.get_untracked() {
                last_step_trigger.set(current_trigger);
                // Force one tick
                if let Some(event) = session.tick() {
                    on_discovery.call(event);
                }
            } 
            // 2. Handle Auto-Play
            else if is_playing.get() {
                let target_tps = speed.get();
                let ms_per_tick = 1000.0 / target_tps;
                
                let mut new_acc = accumulator.get_untracked() + safe_dt;
                
                // Run catch-up loops (limit to 5 per frame to prevent freeze)
                let mut loops = 0;
                while new_acc >= ms_per_tick && loops < 5 {
                    if let Some(event) = session.tick() {
                        on_discovery.call(event);
                    }
                    new_acc -= ms_per_tick;
                    loops += 1;
                }
                accumulator.set(new_acc);
            }

            // Update UI counter
            set_tick_count.set(session.step_count);

            // 3. Draw (Always draw, even if paused, to see the state)
            if let Some(canvas) = canvas_ref.get_untracked() {
                if let Ok(Some(ctx_val)) = canvas.get_context("2d") {
                    if let Ok(ctx) = ctx_val.dyn_into::<CanvasRenderingContext2d>() {
                        draw_simulation(&ctx, &canvas, session.get_state());
                    }
                }
            }
        } else {
             // Clear canvas if no session
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
            style="width: 100%; height: 100%; display: block;" 
        />
    }
}

// --- Draw Functions ---
fn draw_simulation(ctx: &CanvasRenderingContext2d, canvas: &HtmlCanvasElement, state: SimState) {
    let w = canvas.width() as f64;
    let h = canvas.height() as f64;
    ctx.set_fill_style(&"#000".into());
    ctx.fill_rect(0.0, 0.0, w, h);
    match state {
        SimState::Grid { width, height, cells, .. } => draw_grid(ctx, w, h, width, height, &cells),
        SimState::Points(points) => draw_points(ctx, w, h, &points),
        // NEW: Draw the Chemical Soup
        SimState::FloatGrid { width, height, values } => draw_heatmap(ctx, w, h, width, height, &values),
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
