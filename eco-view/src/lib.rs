#![deny(clippy::all, clippy::pedantic)]
// Necessary for Dioxus
#![allow(non_snake_case, clippy::ignored_unit_patterns)]

use std::sync::{Arc, Mutex};
use std::thread::{self, sleep};
use std::time::Duration;

use camino::Utf8PathBuf;
use components::commands::Commands;
use dioxus::desktop::{Config, WindowBuilder};
use dioxus::html::{geometry::WheelDelta, input_data::keyboard_types::Key};
use dioxus::prelude::*;
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

    LaunchBuilder::desktop()
        .with_cfg(desktop!({
            Config::new()
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
                .with_window(WindowBuilder::default().with_title(format!("Eco Viewer - {path}")))
        }))
        .with_context(doc)
        .with_context(MaxPage(max_page))
        .with_context(PageLoadedReceiver(Arc::new(Mutex::new(Some(
            page_loaded_receiver,
        )))))
        .with_context(file_type)
        .launch(app);

    Ok(())
}

#[derive(Clone)]
struct PageLoadedReceiver(Arc<Mutex<Option<mpsc::UnboundedReceiver<()>>>>);

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
struct MaxPage(usize);

fn load_pages<F>(
    doc: SharedDoc,
    max_page: usize,
    mut page_loaded_sender: UnboundedSender<()>,
    done: F,
) where
    F: Send + FnOnce() + 'static,
{
    thread::spawn(move || {
        for page in 1..=max_page {
            let mut doc = doc.lock().unwrap();
            if let Err(err) = doc.load_page(page) {
                error!("page load failed: {err}");
            }
            drop(doc);
            // Gives some breath to the UI
            sleep(Duration::from_millis(1));
            if let Err(err) = block_on(page_loaded_sender.send(())) {
                error!("page loaded channel error: {err}");
            }
        }
        done();
    });
}

fn app() -> Element {
    let doc = use_context::<SharedDoc>();
    let MaxPage(max_page) = use_context::<MaxPage>();
    let PageLoadedReceiver(page_loaded_receiver) = use_context::<PageLoadedReceiver>();

    let mut nb_loaded_pages = use_signal(|| 0);
    let mut current_page = use_signal(|| 1_usize);

    let current_content = use_memo({
        let doc = doc.clone();
        move || {
            let doc = doc.lock().unwrap();
            doc.content_for_page(current_page())
        }
    });

    spawn(async move {
        let mut page_loaded_receiver = page_loaded_receiver.lock().unwrap().take();
        if let Some(page_loaded_receiver) = page_loaded_receiver.as_mut() {
            while page_loaded_receiver.next().await.is_some() {
                nb_loaded_pages += 1;
            }
        }
    });

    let mut go_to_prev_page = move || {
        if current_page() == 1 {
            return;
        }
        current_page -= 1;
        debug!("reading index {current_page}");
    };

    let mut go_to_next_page = move || {
        if current_page() == nb_loaded_pages() {
            return;
        }
        current_page += 1;
        debug!("reading index {current_page}");
    };

    let handle_wheel_events = move |evt: Event<WheelData>| {
        let delta = match evt.delta() {
            WheelDelta::Pixels(px) => px.y,
            WheelDelta::Lines(lines) => lines.y,
            WheelDelta::Pages(pages) => pages.y,
        };
        if delta < 0.0 {
            go_to_prev_page();
        } else {
            go_to_next_page();
        }
    };

    let handle_keyup_events = move |evt: Event<KeyboardData>| match evt.key() {
        Key::ArrowLeft | Key::ArrowUp => go_to_prev_page(),
        Key::ArrowRight | Key::ArrowDown => go_to_next_page(),
        _ => {}
    };

    rsx! {
        div {
            class: "w-full h-screen flex flex-col gap-1 items-center outline-none",
            autofocus: true,
            tabindex: -1,
            onwheel: handle_wheel_events,
            onkeyup: handle_keyup_events,
            div { class: "relative h-2 w-full shrink-0 px-2 mt-1",
                if nb_loaded_pages() < max_page {
                    Progress { max_page, nb_loaded_pages }
                }
            }
            div { class: "flex flex-col h-full w-full items-center justify-center",
                if let Some(content) = current_content() {
                    DocPage { content }
                }
            }
            Commands {
                max_page,
                nb_loaded_pages,
                current_page,
                on_prev_page_request: move |()| go_to_prev_page(),
                on_next_page_request: move |()| go_to_next_page()
            }
        }
    }
}

#[component]
fn Progress(max_page: usize, nb_loaded_pages: ReadOnlySignal<usize>) -> Element {
    #[allow(clippy::cast_precision_loss)]
    let progress = use_memo(move || 1.0 / (max_page as f32) * (nb_loaded_pages() as f32) * 100.0);

    rsx! {
        progress {
            class: "progress progress-flat-primary absolute h-2 w-[calc(100%-1rem)]",
            value: "{progress}",
            max: "100"
        }
    }
}
