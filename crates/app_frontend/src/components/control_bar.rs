use leptos::*;

#[component]
pub fn ControlBar(
    /// Whether the simulation is currently running
    is_playing: ReadSignal<bool>,
    /// Toggle play/pause
    set_playing: WriteSignal<bool>,
    /// Ticks per second (speed)
    speed: ReadSignal<f64>,
    /// Update speed
    set_speed: WriteSignal<f64>,
    /// Callback to trigger a reset
    on_reset: Callback<()>,
    /// Callback to step forward exactly one tick (only works when paused)
    on_step: Callback<()>,
    /// Current tick count to display
    tick_count: ReadSignal<u64>,
) -> impl IntoView {
    view! {
        <div style="
            background: #2a2a2a;
            padding: 1rem;
            border-top: 1px solid #444;
            display: flex;
            align_items: center;
            gap: 1.5rem;
            color: #e0e0e0;
            font-size: 0.9rem;
        ">
            // Play / Pause
            <button
                on:click=move |_| set_playing.update(|p| *p = !*p)
                style="width: 80px; cursor: pointer;"
            >
                {move || if is_playing.get() { "PAUSE" } else { "PLAY" }}
            </button>

            // Step (Only enabled if paused)
            <button
                on:click=move |_| on_step.call(())
                disabled=move || is_playing.get()
                style=move || format!("opacity: {}; cursor: pointer;", if is_playing.get() { "0.5" } else { "1.0" })
            >
                "STEP >"
            </button>

            // Reset
            <button
                on:click=move |_| on_reset.call(())
                style="background-color: #cc3300; cursor: pointer;"
            >
                "RESET"
            </button>

            // Speed Slider
            <div style="display: flex; align_items: center; gap: 0.5rem;">
                <span>"Speed:"</span>
                <input
                    type="range"
                    min="1"
                    max="60"
                    step="1"
                    prop:value=move || speed.get()
                    on:input=move |ev| {
                        if let Ok(val) = event_target_value(&ev).parse::<f64>() {
                            set_speed.set(val);
                        }
                    }
                    style="cursor: grab;"
                />
                <span style="min-width: 3ch; text-align: right;">
                    {move || speed.get()}
                </span>
                <span>" tps"</span>
            </div>

            // Info Stats
            <div style="margin-left: auto; font-family: monospace; color: #00aaff;">
                "Tick: " {move || tick_count.get()}
            </div>
        </div>
    }
}
