use dioxus::prelude::*;

use crate::FileType;

#[component]
pub fn DocPage(content: ReadOnlySignal<String>) -> Element {
    let file_type = use_context::<FileType>();

    match file_type {
        FileType::Cbz => rsx!(img {
            class: "h-px grow",
            src: "data:image/png;base64,{content}"
        }),
        FileType::Epub => rsx!(
            div { class: "h-px grow spect-[12/16]",
                iframe {
                    class: "h-full w-full",
                    src: "data:text/html;charset=utf-8,{content}"
                }
            }
        ),
    }
}
