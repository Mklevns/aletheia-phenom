use leptos::*;
use sim_engine::gol::GameOfLife; // Specific imports
use sim_engine::ode::ODESim;
use sim_engine::Simulation; // Import the trait

/// The global application root.
/// This defines a top-level signal containing a boxed simulation.
#[component]
pub fn App() -> impl IntoView {
    // A writable signal that holds our active simulation.
    // It begins as None until a system is loaded.
    let active_sim: RwSignal<Option<Box<dyn Simulation>>> = create_rw_signal(None);

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

    view! {
        <main>
            <h1>"Aletheia-Phenom — v0.01: 'Hello Chaos'"</h1>

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

            // We will replace this debug output with the <SimulationViewport/>
            <Show
                when=move || active_sim.get().is_some()
                fallback=|| view! { <p>"No simulation loaded yet."</p> }
            >
                {move || {
                    let sim_is_loaded = active_sim.get().is_some();
                    if sim_is_loaded {
                        view! {
                            <p style="color: #00ff00; font-style: italic;">
                                "Simulation loaded."
                            </p>
                        }
                        .into_view()
                    } else {
                         view! { <p>"No simulation loaded yet."</p> }.into_view()
                    }
                }}
            </Show>
        </main>
    }
}
