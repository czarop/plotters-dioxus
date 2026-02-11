#![allow(non_snake_case)]

use clingate::plotters_dioxus::{plot_window::PlotWindow, route::Route};
use dioxus::{
    desktop::{Config, LogicalSize, WindowBuilder}, prelude::*
};


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

fn main() {
    LaunchBuilder::new()
        .with_cfg(
            Config::new().with_window(
                WindowBuilder::new()
                    .with_title("FCS Plot Viewer")
                    .with_always_on_top(false)
                    .with_inner_size(LogicalSize::new(1200.0, 900.0)),
            ),
        )
        .launch(App);
}

