use leptos::*;
use sim_engine::gol::GameOfLife;
use sim_engine::ode::ODESim;
use sim_engine::gray_scott::GrayScott;
// UPDATED IMPORTS: Added create_brain and BrainType
use inference_engine::{DiscoveryEvent, create_brain, BrainType};

mod components;
pub mod session;

use crate::components::discovery_feed::DiscoveryFeed;
use crate::components::simulation_viewport::SimulationViewport;
use crate::components::control_bar::ControlBar;
use crate::session::Session;

#[component]
pub fn App() -> impl IntoView {
    // Session State
    let active_session: RwSignal<Option<Session>> = create_rw_signal(None);
    let history: RwSignal<Vec<DiscoveryEvent>> = create_rw_signal(Vec::new());
    
    // Control State
    let is_playing = create_rw_signal(false); 
    let speed = create_rw_signal(10.0);       
    let tick_count = create_rw_signal(0);

    // Helper to store "which" sim is loaded so we can reset it
    let (current_sim_type, set_sim_type) = create_signal("none");

    // --- Loaders ---
    let load_gol = {
        let active_session = active_session.clone();
        move |_| {
            let sim = Box::new(GameOfLife::new());
            // UPDATED: Now uses the Gardener Brain
            let agent = create_brain(BrainType::Gardener); 
            active_session.set(Some(Session::new(sim, agent)));
            set_sim_type.set("gol");
            tick_count.set(0);
            is_playing.set(true); // Auto-play on load
        }
    };

    let load_lorenz = {
        let active_session = active_session.clone();
        move |_| {
            let sim = Box::new(ODESim::new());
            // UPDATED: Now uses the Q-Learning Brain
            let agent = create_brain(BrainType::QLearner); 
            active_session.set(Some(Session::new(sim, agent)));
            set_sim_type.set("ode");
            tick_count.set(0);
            is_playing.set(true);
        }
    };
    
    let load_gs = {
        let active_session = active_session.clone();
        move |_| {
            let sim = Box::new(GrayScott::init(100, 100));
            // Use the Gardener agent (it likes to water/feed things)
            let agent = create_brain(BrainType::Gardener); 
            active_session.set(Some(Session::new(sim, agent)));
            set_sim_type.set("gs");
            tick_count.set(0);
            is_playing.set(true);
        }
    };
    
    // --- Handlers ---
    let on_reset = move |_| {
        match current_sim_type.get() {
            "gol" => load_gol(()),
            "ode" => load_lorenz(()),
            _ => {}
        }
    };

    // We use a simple counter signal to trigger single steps in the Viewport
    let (step_trigger, set_step_trigger) = create_signal(0); 
    let on_step = move |_| set_step_trigger.update(|n| *n += 1);

    view! {
        <main style="display: flex; width: 100%; height: 100vh; flex-direction: column;">
            <div style="flex: 1; display: flex; overflow: hidden;">
                // --- LEFT COLUMN ---
                <div class="main-content" style="flex: 3; display: flex; flex-direction: column; border-right: 1px solid #444;">
                    
                    // Header / Toolbar
                    <div style="padding: 1rem; border-bottom: 1px solid #444;">
                        <h1 style="margin: 0 0 1rem 0; font-size: 1.5rem; color: #00aaff;">"Aletheia-Phenom"</h1>
                        <div>
                            <button on:click=load_gol>"Load Game of Life"</button>
                            <button on:click=load_lorenz>"Load Lorenz"</button>
                        </div>
                    </div>

                    // Viewport (Fills remaining space)
                    <div style="flex: 1; position: relative; background: #000;">
                        <SimulationViewport 
                            active_session=active_session
                            is_playing=is_playing.read_only()
                            speed=speed.read_only()
                            step_trigger=step_trigger.into()
                            set_tick_count=tick_count.write_only()
                            on_discovery=move |evt| {
                                history.update(|h| {
                                    h.push(evt);
                                    if h.len() > 50 { h.remove(0); }
                                });
                            }
                        />
                    </div>

                    // Control Bar (Bottom)
                    <ControlBar 
                        is_playing=is_playing.read_only()
                        set_playing=is_playing.write_only()
                        speed=speed.read_only()
                        set_speed=set_speed.write_only()
                        on_reset=on_reset
                        on_step=on_step
                        tick_count=tick_count.read_only()
                    />
                </div>

                // --- RIGHT COLUMN (Sidebar) ---
                <div class="sidebar" style="flex: 1; background-color: #2a2a2a; overflow-y: auto; border-left: 1px solid #444;">
                    <DiscoveryFeed history=history.read_only() />
                </div>
            </div>
        </main>
    }
}
