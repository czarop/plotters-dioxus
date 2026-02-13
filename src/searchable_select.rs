use dioxus::prelude::*;

#[component]
pub fn SearchableSelect<T: Clone + PartialEq + std::fmt::Display + 'static>(
    items: ReadSignal<Vec<T>>,
    on_select: EventHandler<(usize, T)>,
    placeholder: Option<String>,
    selected_index: Option<ReadSignal<usize>>,
) -> Element {
    // In your component
    let mut is_open = use_signal(|| false);
    let mut search = use_signal(|| String::new());
    let mut local_selected_value: Signal<Option<(usize, T)>> = use_signal(|| None);

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

    let mut display_text = use_signal(|| String::new());

    use_effect(move || {
        let text = local_selected_value
            .read()
            .as_ref()
            .map(|(_, v)| v.to_string())
            .or(placeholder.clone())
            .unwrap_or_else(|| "Select an item...".to_string());
        display_text.set(text);
    });

    use_effect(move || {
        if let Some((i, val)) = local_selected_value() {
            on_select((i, val));
        };
    });

    use_effect(move || {
        if let Some(sig) = selected_index {
            let i = sig();
            let items = &*items.peek();
            if i < items.len() {
                display_text.set(items[i].to_string())
            }
        }
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
                                        local_selected_value.set(Some((i, current_item.clone())));

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
