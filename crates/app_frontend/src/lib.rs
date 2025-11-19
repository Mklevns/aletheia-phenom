use leptos::*;
use sim_engine::gol::GameOfLife;
use sim_engine::ode::ODESim;
use sim_engine::Simulation;
use inference_engine::{DiscoveryEvent, get_mock_inference};

mod components;
use crate::components::discovery_feed::DiscoveryFeed;
use crate::components::simulation_viewport::SimulationViewport;

#[component]
pub fn App() -> impl IntoView {
    let active_sim: RwSignal<Option<Box<dyn Simulation>>> = create_rw_signal(None);
    let history: RwSignal<Vec<DiscoveryEvent>> = create_rw_signal(Vec::new());

    // Callbacks for loading different simulators.
    let load_gol = {
        let active_sim = active_sim.clone();
        move |_| {
            active_sim.set(Some(Box::new(GameOfLife::new())));
        }
    };

    let load_lorenz = {
        let active_sim = active_sim.clone();
        move |_| {
            active_sim.set(Some(Box::new(ODESim::new())));
        }
    };
    
    // Poll for mock events (temporary wiring)
    request_animation_frame_loop(move || {
        // Use a pseudo-tick based on time or frame count if we had one globally.
        // For now, we just use random chance to simulate "occasional" events from the engine
        if js_sys::Math::random() < 0.01 {
             let tick = (js_sys::Date::now() as u64) / 1000;
             if let Some(evt) = get_mock_inference(tick) {
                 history.update(|h| {
                     h.push(evt);
                     if h.len() > 50 { h.remove(0); }
                 });
             }
        }
    });

    view! {
        <main
            style="
                display: flex;
                width: 100%;
                height: 100%;
            "
        >
            // --- LEFT COLUMN (MAIN CONTENT) ---
            <div
                class="main-content"
                style="
                    flex: 3;
                    padding: 2rem;
                    overflow-y: auto;
                    height: 100vh;
                    box-sizing: border-box;
                "
            >
                <h1>"Aletheia-Phenom — Live Universe"</h1>
                <p>"Select a simulation to begin:"</p>
                <div>
                    <button on:click=load_gol>
                        "Load Game of Life"
                    </button>
                    <button on:click=load_lorenz>
                        "Load Lorenz Attractor"
                    </button>
                </div>
                <hr/>
                <SimulationViewport active_sim=active_sim/>
            </div>

            // --- RIGHT COLUMN (SIDEBAR) ---
            <div
                class="sidebar"
                style="
                    flex: 1;
                    border-left: 1px solid #444;
                    background-color: #2a2a2a;
                    height: 100vh;
                    overflow-y: auto;
                    box-sizing: border-box;
                "
            >
                <DiscoveryFeed history=history.read_only() />
            </div>
        </main>
    }
}
