#![deny(clippy::all, clippy::pedantic)]
// Necessary for Dioxus
#![allow(non_snake_case, clippy::ignored_unit_patterns)]

use std::{cell::Cell, thread};

use camino::Utf8PathBuf;
use dioxus::{
    html::{geometry::WheelDelta, input_data::keyboard_types::Key},
    prelude::*,
};
use dioxus_desktop::{Config, WindowBuilder};
use doc::try_load_shared_doc_from_path;
use futures::{channel::mpsc, executor::block_on, SinkExt, StreamExt};
use tracing::{debug, error};

use crate::components::doc_page::DocPage;
pub use crate::doc::FileType;
use crate::doc::SharedDoc;
pub use crate::errors::{Error, Result};

mod components;
mod doc;
pub mod errors;
mod measure;

fn load_pages<F>(
    doc: SharedDoc,
    max_page: usize,
    mut page_loaded_sender: UnboundedSender<()>,
    done: F,
) where
    F: 'static + Send + FnOnce(),
{
    thread::spawn(move || {
        for page in 1..=max_page {
            let mut doc = doc.lock().unwrap();
            if let Err(err) = doc.load_page(page) {
                error!("page load failed: {err}");
            }
            drop(doc);
            // Gives some breath to the UI
            std::thread::sleep(std::time::Duration::from_millis(1000 / 60));
            if let Err(err) = block_on(page_loaded_sender.send(())) {
                error!("page loaded channel error: {err}");
            }
        }
        done();
    });
}

#[derive(Debug)]
pub struct ViewOptions {
    /// The path to the e-book file to view
    pub path: Utf8PathBuf,

    /// Type of the file
    pub type_: Option<FileType>,
}

/// Starts a new window with the viewer inside
///
/// ## Errors
///
/// Fails on file read error
///
/// ## Panics
pub fn view(opts: ViewOptions) -> Result<()> {
    let Ok(path) = Utf8PathBuf::try_from(dunce::canonicalize(opts.path)?) else {
        return Err(Error::InvalidNonUtf8Path);
    };
    let Some(file_type) = opts
        .type_
        .or_else(|| path.extension().and_then(|ext| ext.parse().ok()))
    else {
        return Err(Error::UnknownFileType);
    };

    let path = path.as_ref();
    let (max_page, doc) = try_load_shared_doc_from_path(file_type, path)?;
    let (page_loaded_sender, page_loaded_receiver) = mpsc::unbounded::<()>();
    let measure =
        crate::measure::Measure::new("total document loading time", crate::measure::Precision::Ms);

    load_pages(doc.clone(), max_page, page_loaded_sender, move || {
        drop(measure);
    });

    dioxus_desktop::launch_with_props(
        App,
        AppProps {
            doc,
            max_page,
            page_loaded_receiver: Cell::new(Some(page_loaded_receiver)),
        },
        Config::default()
            .with_custom_head(
                r#"
                    <link
                        rel="stylesheet"
                        href="https://cdn.jsdelivr.net/npm/rippleui@1.12.1/dist/css/styles.css"
                    />
                    <script src="https://cdn.tailwindcss.com"></script>
                "#
                .to_string(),
            )
            .with_window(WindowBuilder::default().with_title(format!("Eco Viewer - {path}"))),
    );

    Ok(())
}

pub struct AppProps {
    doc: SharedDoc,
    max_page: usize,
    // Wrapped in an `Option` so it can be moved out from the struct
    page_loaded_receiver: Cell<Option<mpsc::UnboundedReceiver<()>>>,
}

#[allow(clippy::too_many_lines)]
fn App(cx: Scope<AppProps>) -> Element {
    let page_loaded_receiver = cx.props.page_loaded_receiver.replace(None);
    // Forces reactivity on page loaded
    let nb_loaded_pages = use_state(cx, || 0);
    let current_page = use_state(cx, || 1_usize);
    #[allow(clippy::cast_precision_loss)]
    let progress = use_memo(cx, (nb_loaded_pages,), |(nb_loaded_pages,)| {
        1.0 / (cx.props.max_page as f32) * (*nb_loaded_pages.get() as f32) * 100.0
    });
    let current_content = use_memo(
        cx,
        (current_page, nb_loaded_pages),
        |(current_page, _nb_loaded_pages)| {
            let doc = cx.props.doc.lock().unwrap();
            doc.content_for_page(*current_page.get())
        },
    );

    use_future!(cx, || {
        to_owned![nb_loaded_pages];
        async move {
            let mut page_loaded_receiver =
                page_loaded_receiver.expect("page loaded receiver to be accessed once");
            while page_loaded_receiver.next().await.is_some() {
                nb_loaded_pages.modify(|nb_loaded_pages| *nb_loaded_pages + 1);
            }
        }
    });

    cx.render(rsx! {
        div {
            class: "w-full h-screen flex flex-col gap-1 items-center outline-none",
            autofocus: true,
            tabindex: -1,
            onwheel: move |evt| {
                let delta = match evt.delta() {
                    WheelDelta::Pixels(px) => px.y,
                    WheelDelta::Lines(lines) => lines.y,
                    WheelDelta::Pages(pages) => pages.y,
                };
                if delta < 0.0 {
                    let page = *current_page.get();
                    if page == 1 {
                        return;
                    }

                    current_page.set(page - 1);
                    debug!("reading index {}", page - 2);
                } else {
                    let page = *current_page.get();
                    if page == *nb_loaded_pages.get() {
                        return;
                    }

                    current_page.set(page + 1);
                    debug!("reading index {}", page);
                }
            },
            onkeyup: move |evt| {
                match evt.key() {
                    Key::ArrowLeft | Key::ArrowUp => {
                        let page = *current_page.get();
                        if page == 1 {
                            return;
                        }

                        current_page.set(page - 1);
                        debug!("reading index {}", page - 2);
                    },
                    Key::ArrowRight | Key::ArrowDown => {
                        let page = *current_page.get();
                        if page == *nb_loaded_pages.get() {
                            return;
                        }

                        current_page.set(page + 1);
                        debug!("reading index {}", page);
                    },
                    _ => {}
                }
            },
            div {
                class: "relative h-2 w-full shrink-0 px-2 mt-1",
                if *nb_loaded_pages.get() < cx.props.max_page  {
                    rsx!(progress {
                        class: "progress progress-flat-primary absolute h-2 w-[calc(100%-1rem)]",
                        value: "{progress}",
                        max: "100"
                    })
                }
            }
            div {
                class: "flex flex-col h-full w-full items-center justify-center",
                if let Some(current_content) = current_content {
                    rsx!(DocPage { doc: cx.props.doc.clone(), content: current_content })
                }
            }
            div {
                class: "flex flex-row items-center justify-center gap-1 h-8 mb-2",
                button {
                    class: "btn btn-outline-primary btn-sm",
                    onclick: move |_evt| {
                        let page = *current_page.get();
                        if page == 1 {
                            return;
                        }

                        current_page.set(page - 1);
                        debug!("reading index {}", page - 2);
                    },
                    "Prev"
                },
                span {
                    class: "flex flex-row items-center justify-center bg-backgroundSecondary h-8 px-2 rounded-sm",
                     "{current_page} / {nb_loaded_pages}"
                },
                button {
                    class: "btn btn-outline-primary btn-sm",
                    onclick: move |_evt| {
                        let page = *current_page.get();
                        if page == *nb_loaded_pages.get() {
                            return;
                        }

                        current_page.set(page + 1);
                        debug!("reading index {}", page);
                    },
                    "Next"
                },
            }
        }
    })
}
