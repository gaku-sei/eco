use dioxus::prelude::*;

use crate::doc::{Doc, SharedDoc};

#[allow(clippy::module_name_repetitions)]
#[derive(Props)]
pub struct DocPageProps<'a> {
    doc: SharedDoc,
    content: &'a str,
}

pub fn DocPage<'a, 'b: 'a>(cx: Scope<'a, DocPageProps<'b>>) -> Element<'a> {
    let content = cx.props.content;

    match *cx.props.doc.read().unwrap() {
        Doc::Cbz { .. } => cx.render(rsx!(img {
            class: "h-px grow",
            src: "data:image/png;base64,{content}"
        })),
        Doc::Epub { .. } => cx.render(rsx!(div {
            class: "h-px grow spect-[12/16]",
            iframe {
                class: "h-full w-full",
                src: "data:text/html;charset=utf-8,{content}"
            }
        })),
    }
}
