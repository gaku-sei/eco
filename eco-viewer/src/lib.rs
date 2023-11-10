#![deny(clippy::all)]
#![deny(clippy::pedantic)]
#![allow(non_snake_case)]

use std::{
    fs::File,
    io::{BufReader, Read, Seek},
    str::FromStr,
};

use base64::Engine;
use camino::Utf8Path;
use clap::ValueEnum;
use dioxus::{html::input_data::keyboard_types::Key, prelude::*};
use dioxus_desktop::{Config, WindowBuilder};
use eco_cbz::CbzReader;
use tl::{HTMLTag, ParserOptions, VDom};
use tracing::debug;

#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
pub enum FileType {
    #[clap(name = "cbz")]
    Cbz,
    #[clap(skip, name = "epub")]
    EPub,
}

impl FromStr for FileType {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self> {
        match s.to_lowercase().as_str() {
            "cbz" => Ok(FileType::Cbz),
            "epub" => Ok(FileType::EPub),
            _ => Err(Error::InvalidFileType(s.to_string())),
        }
    }
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("cbz error: {0}")]
    Cbz(#[from] eco_cbz::errors::Error),

    #[error("epub doc error: {0}")]
    EpubDoc(#[from] epub::doc::DocError),

    #[error("zip error: {0}")]
    Zip(#[from] zip::result::ZipError),

    #[error("invalid file type: {0}")]
    InvalidFileType(String),
}

type Result<T, E = Error> = std::result::Result<T, E>;

pub struct AppProps {
    archive: Box<RefCell<dyn Doc>>,
}

pub trait Doc {
    fn load_page(&mut self, page: usize) -> Option<String>;

    fn render_page<'a, 'b>(&mut self, page: usize) -> Option<LazyNodes<'a, 'b>>;

    fn len(&self) -> usize;

    fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

pub struct CbzDoc<T> {
    archive: CbzReader<T>,
    file_names: Vec<String>,
}

impl CbzDoc<File> {
    fn try_from_path(path: &Utf8Path) -> Result<Self> {
        let archive = CbzReader::try_from_path(path)?;
        let file_names = archive.file_names();
        Ok(Self {
            archive,
            file_names,
        })
    }
}

impl Doc for CbzDoc<File> {
    fn load_page(&mut self, page: usize) -> Option<String> {
        let file_name = self.file_names.get(page - 1)?;
        let image = self.archive.read_by_name(file_name.as_str()).ok()?;
        let bytes = image.try_into_bytes().ok()?;
        Some(base64::engine::general_purpose::STANDARD.encode(bytes))
    }

    fn render_page<'a, 'b>(&mut self, page: usize) -> Option<LazyNodes<'a, 'b>> {
        self.load_page(page).map(|content| {
            rsx!(img {
                class: "h-full w-full",
                src: "data:image/png;base64,{content}"
            })
        })
    }

    fn len(&self) -> usize {
        self.archive.len()
    }
}

pub struct EpubDoc<T: Read + Seek> {
    doc: epub::doc::EpubDoc<T>,
}

impl EpubDoc<BufReader<File>> {
    fn try_from_path(path: &Utf8Path) -> Result<Self> {
        Ok(Self {
            doc: epub::doc::EpubDoc::new(path)?,
        })
    }

    fn for_each_tag_mut<F>(dom: &mut VDom, selector: &str, mut f: F)
    where
        F: FnMut(&mut HTMLTag<'_>),
    {
        let Some(node_handles) = dom.query_selector(selector) else {
            debug!("no nodes found");
            return;
        };
        for node_handle in node_handles.collect::<Vec<_>>() {
            let Some(node) = node_handle.get_mut(dom.parser_mut()) else {
                debug!("node not found {}", node_handle.get_inner());
                continue;
            };
            let Some(tag) = node.as_tag_mut() else {
                debug!("node is not a tag {node:#?}");
                continue;
            };
            f(tag);
        }
    }
}

impl Doc for EpubDoc<BufReader<File>> {
    fn load_page(&mut self, page: usize) -> Option<String> {
        self.doc.set_current_page(page - 1);
        let content = self.doc.get_current_with_epub_uris().ok()?;
        let content = String::from_utf8_lossy(&content);
        let mut dom = tl::parse(content.as_ref(), ParserOptions::default()).ok()?;
        Self::for_each_tag_mut(&mut dom, "img", |tag| {
            let Some(Some(src)) = tag.attributes_mut().get_mut("src") else {
                return;
            };
            let Some(res) = self
                .doc
                .get_resource_by_path(&src.as_utf8_str().as_ref()[7..])
            else {
                return;
            };
            if let Some(bytes) = Some(base64::engine::general_purpose::STANDARD.encode(res)) {
                *src = format!("data:image/png;base64,{bytes}").try_into().unwrap();
            }
        });
        Some(dom.outer_html())
    }

    fn render_page<'a, 'b>(&mut self, page: usize) -> Option<LazyNodes<'a, 'b>> {
        self.load_page(page).map(|content| {
            rsx!(div {
                class: "h-full w-full aspect-[11/16]",
                iframe {
                    class: "h-full w-full",
                    src: "data:text/html;charset=utf-8,{content}"
                }
            })
        })
    }

    fn len(&self) -> usize {
        // FIXME: Wrong size?
        self.doc.get_num_pages()
    }
}

/// Starts a new window with the viewer inside
///
/// ## Errors
///
/// Fails on file read error
pub fn run(path: impl AsRef<Utf8Path>, type_: FileType) -> Result<()> {
    let path = path.as_ref();
    let archive: Box<RefCell<dyn Doc>> = match type_ {
        FileType::Cbz => Box::new(RefCell::new(CbzDoc::try_from_path(path)?)),
        FileType::EPub => Box::new(RefCell::new(EpubDoc::try_from_path(path)?)),
    };

    dioxus_desktop::launch_with_props(
        App,
        AppProps { archive },
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

fn App(cx: Scope<AppProps>) -> Element {
    let max_page = use_state(cx, || cx.props.archive.borrow().len());
    let current_page = use_state(cx, || 1_usize);

    cx.render(rsx! {
        div {
            class: "p-2 w-full h-screen flex flex-col gap-1 items-center outline-none",
            autofocus: true,
            tabindex: -1,
            onkeydown: move |evt| {
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
                        if page == *max_page.get() {
                            return;
                        }

                        current_page.set(page + 1);
                        debug!("reading index {}", page);
                    },
                    _ => {}
                }
            },
            div {
                class: "h-[calc(100%-2rem)] shadow-lg",
                cx.props.archive.borrow_mut().render_page(*current_page.get())
            }
            div {
                class: "flex flex-row items-center justify-center gap-1 h-8",
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
                     "{current_page} / {max_page}"
                },
                button {
                    class: "btn btn-outline-primary btn-sm",
                    onclick: move |_evt| {
                        let page = *current_page.get();
                        if page == *max_page.get() {
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
