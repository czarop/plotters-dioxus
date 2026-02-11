use crate::plotters_dioxus::plot_window::PlotWindow;
use dioxus::prelude::*;

#[derive(Routable, Clone, PartialEq)]
pub enum Route {
    #[layout(NavBar)]
    #[route("/")]
    PlotWindow,
    // #[route("/scale")]
    // ScaleWindow,

    // #[route("/comp")]
    // CompWindow,

    // #[route("/options")]
    // OptionsWindow,

    // #[route("/:..segments")]
    // PageNotFound { segments: Vec<String> },
}

#[component]
pub fn NavBar() -> Element {
    // let mut nav_burger_menu_open = use_signal(|| "".to_string());

    rsx! {
        div { class: "route-outlet", Outlet::<Route> {} }
        div { class: "route-nav_bar",
            nav { aria_label: "main navigation", role: "navigation",
                div { class: "nav_bar-items",

                    div {
                        Link { to: Route::PlotWindow,
                            div { class: "nav_bar-item", "🏠" }
                        }
                    }

                    div {
                        div { class: "nav_bar-item", "|" }
                    }

                // div {
                //     if geolocation::check_geolocation_permission() == PermissionResult::GRANTED {
                //         Link { to: route::Route::LocationMap,
                //             div { class: "nav_bar-item", "📍" }
                //         }
                //     } else {
                //         div {
                //             class: "nav_bar-item",
                //             onclick: move |_| {
                //                 geolocation::request_geolocation_permissions();
                //             },
                //             "📍"
                //         }
                //     }
                // }

                // div {
                //     div { class: "nav_bar-item", "|" }
                // }

                // div {
                //     Link { to: route::Route::LoginScreen,
                //         div { class: "nav_bar-item", "🔐" }
                //     }
                // }

                // div {
                //     div { class: "nav_bar-item", "|" }
                // }

                // div {
                //     Link { to: route::Route::ContactScreen,
                //         div { class: "nav_bar-item", "👥" }
                //     }
                // }
                // a {
                //     aria_expanded: "false",
                //     aria_label: "menu",
                //     class: "navbar-burger {nav_burger_menu_open}",
                //     "data-target": "navbarBasicExample",
                //     role: "button",
                //     onclick: move |_| {
                //         let current = nav_burger_menu_open();
                //         if current == "".to_string() {
                //             nav_burger_menu_open.set("is-active".to_string());
                //         } else {
                //             nav_burger_menu_open.set("".to_string());
                //         }
                //     },
                //     span { aria_hidden: "true" }
                //     span { aria_hidden: "true" }
                //     span { aria_hidden: "true" }
                //     span { aria_hidden: "true" }
                // }
                }
                        // div {
            //     class: "navbar-menu {nav_burger_menu_open}",
            //     id: "navbarBasicExample",
            //     div { class: "navbar-start",
            //         a { class: "navbar-item", "Home" }
            //         a { class: "navbar-item", "Documentation" }

            //         div { class: "navbar-item has-dropdown is-hoverable",
            //             a { class: "navbar-link", "More" }

            //             div { class: "navbar-dropdown",
            //                 a { class: "navbar-item", "About" }
            //                 a { class: "navbar-item", "Jobs" }
            //                 a { class: "navbar-item", "Contact" }
            //                 hr { class: "navbar-divider" }
            //                 a { class: "navbar-item", "Report an issue" }
            //             }
            //         }
            //     }
            //     div { class: "navbar-end",
            //         div { class: "navbar-item",
            //             div { class: "buttons",
            //                 a { class: "button is-primary",
            //                     strong { "Sign up" }
            //                 }
            //                 a { class: "button is-light", "Log in" }
            //             }
            //         }
            //     }
            // }
            }
        }
    }
}
