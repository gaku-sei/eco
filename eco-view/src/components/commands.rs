use dioxus::prelude::*;

#[component]
pub fn Commands(
    max_page: usize,
    nb_loaded_pages: ReadOnlySignal<usize>,
    current_page: Signal<usize>,
    on_prev_page_request: EventHandler,
    on_next_page_request: EventHandler,
) -> Element {
    rsx! {
        div { class: "flex flex-row items-center justify-center gap-1 h-8 mb-2",
            button {
                class: "btn btn-outline-primary btn-sm",
                onclick: move |_evt| on_prev_page_request.call(()),
                "Prev"
            }
            span { class: "flex flex-row items-center justify-center bg-backgroundSecondary h-8 px-2 rounded-sm",
                "{current_page} / {nb_loaded_pages}"
            }
            button {
                class: "btn btn-outline-primary btn-sm",
                onclick: move |_evt| on_next_page_request.call(()),
                "Next"
            }
        }
        div { class: "w-full flex flex-row items-center justify-center gap-1 h-8 mb-2",
            input {
                class: "w-11/12",
                r#type: "range",
                min: "1",
                value: "{current_page}",
                max: "{max_page}",
                oninput: move |evt| { current_page.set(evt.value().parse().unwrap_or(1)) }
            }
        }
    }
}
