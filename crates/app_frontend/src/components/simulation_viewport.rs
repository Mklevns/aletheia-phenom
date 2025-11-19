use leptos::*;
use sim_engine::{SimState, Simulation, Experimentable, Action, Observation};
use inference_engine::{MockExperimenter, Experimenter, AgentAction, AgentObservation, DiscoveryEvent};
use wasm_bindgen::JsCast;
use web_sys::{CanvasRenderingContext2d, HtmlCanvasElement};

// Bridge types
fn map_obs(obs: Observation) -> AgentObservation {
    match obs {
        Observation::GridSummary { width, height, .. } => AgentObservation::GridSummary { width, height },
        Observation::StateVec(v) => AgentObservation::StateVec(v),
        _ => AgentObservation::None,
    }
}

fn map_act(act: AgentAction) -> Action {
    match act {
        AgentAction::FlipCell { r, c } => Action::FlipCell { r, c },
        AgentAction::Perturb { which, delta } => Action::Perturb { which, delta },
        AgentAction::Noop => Action::Noop,
    }
}

#[component]
pub fn SimulationViewport(
    active_sim: RwSignal<Option<Box<dyn Simulation>>>,
) -> impl IntoView {
    let canvas_ref = create_node_ref::<HtmlCanvasElement>();
    
    // To do this properly in Leptos without a stored agent:
    // We will just run the "Mock Logic" directly here for the demo.

    request_animation_frame_loop(move || {
        if active_sim.get_untracked().is_some() {
            active_sim.update(|sim_opt| {
                if let Some(sim) = sim_opt.as_mut() {
                    // 1. RL LOOP
                    // Check if sim supports experiments
                    if let Some(exp) = sim.as_experimentable() {
                        let obs = exp.observe();
                        let agent_obs = map_obs(obs);
                        
                        // Simple inline Agent logic (The "MockExperimenter")
                        // (In real app, call agent.act())
                        let action = match agent_obs {
                            AgentObservation::GridSummary { width, height } => {
                                // Randomly flip center
                                if js_sys::Math::random() < 0.05 {
                                     AgentAction::FlipCell { r: height/2, c: width/2 }
                                } else { AgentAction::Noop }
                            },
                            AgentObservation::StateVec(_) => {
                                // Kick chaos
                                if js_sys::Math::random() < 0.02 {
                                    AgentAction::Perturb { which: 0, delta: 1.5 }
                                } else { AgentAction::Noop }
                            },
                            _ => AgentAction::Noop
                        };

                        let sim_act = map_act(action);
                        exp.apply_action(sim_act);
                    }

                    // 2. Step
                    sim.step();
                }
            });

            // 3. Draw
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
             // clear canvas
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
            style="border: 1px solid #444; background: black; display: block; margin: 2rem auto; width: 80%; max-width: 800px;"
        />
    }
}

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
