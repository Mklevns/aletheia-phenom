use leptos::*;
use inference_engine::DiscoveryEvent;

#[component]
pub fn DiscoveryFeed(
    history: ReadSignal<Vec<DiscoveryEvent>>
) -> impl IntoView {
    view! {
        <div class="discovery-feed" style="padding: 1.5rem; font-size: 0.9rem; height: 100%; box-sizing: border-box; color: #e0e0e0;">
            <h2 style="color: #00aaff; font-weight: 300; margin-top: 0; border-bottom: 1px solid #444; padding-bottom: 0.5rem; margin-bottom: 1rem;">
                "Discovery Feed"
            </h2>
            
            <p style="font-style: italic; color: #666; font-size: 0.8rem;">
                "Inspector Status: MOCK (Dev Mode)"
            </p>
            
            <ul style="list-style: none; padding: 0; margin: 0;">
                <For
                    each=move || {
                        let mut events = history.get();
                        events.reverse();
                        events
                    }
                    key=|event| format!("{:?}", event)
                    children=move |event| {
                        match event {
                            DiscoveryEvent::Text(msg) => view! {
                                <li style="padding: 0.5rem 0; border-bottom: 1px solid #333; color: #aaa;">
                                    {msg}
                                </li>
                            }.into_view(),
                            
                            DiscoveryEvent::ObjectDetection { label, confidence } => view! {
                                <li style="padding: 0.5rem 0; border-bottom: 1px solid #333; color: #00ffcc;">
                                    <span style="font-weight: bold;">"DETECTION: "</span>
                                    {label}
                                    <span style="float: right; opacity: 0.7;">{format!("{:.0}%", confidence * 100.0)}</span>
                                </li>
                            }.into_view(),

                            // --- NEW: HANDLER FOR INSIGHTS ---
                            DiscoveryEvent::Insight { topic, content } => view! {
                                <li style="padding: 0.5rem 0; border-bottom: 1px solid #333; border-left: 3px solid #a0f; padding-left: 0.5rem; margin-top: 0.5rem; color: #e0d0ff;">
                                    <div style="font-size: 0.75rem; text-transform: uppercase; color: #a0f; font-weight: bold; margin-bottom: 0.2rem;">
                                        "HYPOTHESIS: " {topic}
                                    </div>
                                    <div style="font-style: italic;">
                                        "“" {content} "”"
                                    </div>
                                </li>
                            }.into_view()
                        }
                    }
                />
            </ul>
        </div>
    }
}
