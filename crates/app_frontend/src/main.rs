use app_frontend::App;
use leptos::*;

pub fn main() {
    // Initialize panic hook for better errors in WASM.
    console_error_panic_hook::set_once();

    // Mount the App component into <body>.
    mount_to_body(|| {
        view! { <App/> }
    });
}
