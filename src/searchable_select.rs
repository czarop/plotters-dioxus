use dioxus::prelude::*;

#[component]
pub fn SearchableSelect<
    T: Clone + PartialEq + std::fmt::Display + 'static + Into<F>,
    F: Clone + PartialEq + std::fmt::Display + 'static,
>(
    items: ReadSignal<Vec<T>>,
    selected_value: Signal<F>,
    placeholder: Option<String>,
) -> Element {
    // In your component
    let mut is_open = use_signal(|| false);
    let mut search = use_signal(|| String::new());
    let mut local_selected_value: Signal<Option<T>> = use_signal(|| None);

    // Filter logic
    let filtered = use_memo(move || {
        let q = search.read().to_lowercase();
        items
            .read()
            .iter()
            .filter(|item| item.to_string().to_lowercase().contains(&q))
            .cloned()
            .collect::<Vec<T>>()
    });

    let display_text = use_memo(move || {
        local_selected_value
            .read()
            .as_ref()
            .map(|v| v.to_string())
            .or(placeholder.clone())
            .unwrap_or_else(|| "Select an item...".to_string())
    });

    use_effect(move || {
        if let Some(val) = local_selected_value() {
            selected_value.set(val.into());
        };
    });

    rsx! {
        div {
            class: "combobox-container",
            // Close when clicking outside (you might need a window listener for robust closing)
            onmouseleave: move |_| is_open.set(false),

            // The Trigger / Input
            div { class: "combobox-input-wrapper",
                input {
                    class: "combobox-input",
                    value: "{search}",
                    placeholder: "{display_text}", // Show selected if empty

                    autocorrect: "off",
                    autocapitalize: "none",
                    autocomplete: "off",
                    spellcheck: "false",

                    // Open on click or focus
                    onfocus: move |_| is_open.set(true),
                    onclick: move |_| is_open.set(true),

                    // Handle typing
                    oninput: move |e| {
                        search.set(e.value());
                        is_open.set(true);
                    },
                }
                // Optional: Arrow icon
                div {
                    class: "combobox-arrow",
                    onclick: move |_| is_open.toggle(),
                    "▼"
                }
            }

            // The Dropdown List
            // Only render if open
            if is_open() {
                div { class: "combobox-list scrollable",
                    // Iterate directly over items. We use enumerate just for the key.
                    for (i , item) in filtered().into_iter().enumerate() {
                        // Capture the item for the closure
                        {
                            let current_item = item.clone();
                            rsx! {
                                div {
                                    key: "{i}", // React/Dioxus likes keys
                                    class: "combobox-option", // Use the Display implementation
                                    onclick: move |_| {
                                        // Set the actual item, not the index
                                        local_selected_value.set(Some(current_item.clone()));

                                        // Reset search so the placeholder (selected value) shows again
                                        search.set(String::new());
                                        is_open.set(false);
                                    },
                                    // Use the Display implementation
                                    "{item}"
                                }
                            }
                        }
                    }
                    // Handle empty state
                    if filtered.read().len() == 0 {
                        div { class: "combobox-empty", "No results" }
                    }
                }
            }
        }
    }
}
