#![allow(non_snake_case)]

use clingate::gate_editor::route::Route;
use dioxus::prelude::*;

use dioxus::desktop::{Config, LogicalSize, WindowBuilder};

static NAV_STYLE: Asset = asset!("assets/navbar.css");
static COMPONENTS_STYLE: Asset = asset!("assets/dx-components-theme.css");
static COMPONENTS_STYLE_2: Asset = asset!("assets/searchable_select.css");

#[component]
fn App() -> Element {
    rsx! {
        document::Stylesheet { href: COMPONENTS_STYLE }
        document::Stylesheet { href: COMPONENTS_STYLE_2 }
        document::Stylesheet { href: NAV_STYLE }
        div { class: "main_div", Router::<Route> {} }
    }
}

// #[cfg(feature = "desktop")]
fn main() {
    unsafe { std::env::set_var("RUST_BACKTRACE", "1") };
    dioxus::LaunchBuilder::new()
        .with_cfg(
            Config::new()
            .with_disable_context_menu(true)
            .with_window(
                WindowBuilder::new()
                    .with_title("FCS Plot Viewer")
                    .with_inner_size(LogicalSize::new(1500.0, 900.0)),
                    
            ),
        )
        .launch(App);
}

// #[cfg(not(feature = "desktop"))]
// fn main() {
//     // For Web, we use the simple launch
//     // Dioxus handles the WASM panic hook and browser mounting automatically
//     dioxus::launch(App);
// }
