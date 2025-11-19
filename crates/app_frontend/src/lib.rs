use leptos::*;
use sim_engine::gol::GameOfLife;
use sim_engine::ode::ODESim;
use inference_engine::{DiscoveryEvent, MockExperimenter};

mod components;
pub mod session; // Register the new module

use crate::components::discovery_feed::DiscoveryFeed;
use crate::components::simulation_viewport::SimulationViewport;
use crate::session::Session;

#[component]
pub fn App() -> impl IntoView {
    // "active_session" holds both the Physics and the Agent
    let active_session: RwSignal<Option<Session>> = create_rw_signal(None);
    let history: RwSignal<Vec<DiscoveryEvent>> = create_rw_signal(Vec::new());

    // Handler for events coming from the session
    let on_discovery = move |evt: DiscoveryEvent| {
        history.update(|h| {
            h.push(evt);
            if h.len() > 50 { h.remove(0); }
        });
    };

    // Callbacks for loading different lab setups.
    let load_gol = {
        let active_session = active_session.clone();
        move |_| {
            let sim = Box::new(GameOfLife::new());
            let agent = Box::new(MockExperimenter::new()); 
            active_session.set(Some(Session::new(sim, agent)));
        }
    };

    let load_lorenz = {
        let active_session = active_session.clone();
        move |_| {
            let sim = Box::new(ODESim::new());
            let agent = Box::new(MockExperimenter::new()); 
            active_session.set(Some(Session::new(sim, agent)));
        }
    };
    
    // Note: The "polling loop" is gone. 
    // The Session ticks inside SimulationViewport, and triggers 'on_discovery' when needed.

    view! {
        <main
            style="display: flex; width: 100%; height: 100%;"
        >
            // --- LEFT COLUMN (MAIN CONTENT) ---
            <div
                class="main-content"
                style="flex: 3; padding: 2rem; overflow-y: auto; height: 100vh; box-sizing: border-box;"
            >
                <h1>"Aletheia-Phenom — Live Universe"</h1>
                <p>"Select a simulation to begin:"</p>
                <div>
                    <button on:click=load_gol>
                        "Load Game of Life + Agent"
                    </button>
                    <button on:click=load_lorenz>
                        "Load Lorenz Attractor + Agent"
                    </button>
                </div>
                <hr/>
                // Pass the session and the event handler down
                <SimulationViewport 
                    active_session=active_session 
                    on_discovery=on_discovery
                />
            </div>

            // --- RIGHT COLUMN (SIDEBAR) ---
            <div
                class="sidebar"
                style="flex: 1; border-left: 1px solid #444; background-color: #2a2a2a; height: 100vh; overflow-y: auto; box-sizing: border-box;"
            >
                <DiscoveryFeed history=history.read_only() />
            </div>
        </main>
    }
}
