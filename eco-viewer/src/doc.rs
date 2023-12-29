use std::{
    fs::File,
    io::BufReader,
    str::FromStr,
    sync::{Arc, Mutex},
};

use base64::Engine;
use camino::Utf8Path;
use clap::ValueEnum;
use eco_cbz::CbzReader;
use tl::{HTMLTag, ParserOptions, VDom};
use tracing::debug;

use crate::errors::{Error, Result};

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

pub enum Doc {
    Cbz {
        archive: CbzReader<File>,
        max_page: usize,
        file_names: Vec<String>,
        pages: Vec<String>,
    },
    Epub {
        doc: epub::doc::EpubDoc<BufReader<File>>,
        max_page: usize,
        pages: Vec<String>,
    },
}

impl Doc {
    /// ## Errors
    pub fn try_load_from_path(type_: FileType, path: &Utf8Path) -> Result<Doc> {
        match type_ {
            FileType::Cbz => {
                let archive = CbzReader::try_from_path(path)?;
                let file_names = archive.file_names();
                let max_page = file_names.len();
                Ok(Doc::Cbz {
                    archive,
                    file_names,
                    max_page,
                    pages: Vec::with_capacity(max_page),
                })
            }
            FileType::EPub => {
                let doc = epub::doc::EpubDoc::new(path)?;
                let max_page = doc.get_num_pages();
                Ok(Doc::Epub {
                    doc,
                    max_page,
                    pages: Vec::with_capacity(max_page),
                })
            }
        }
    }

    /// ## Errors
    pub fn load_page(&mut self, page: usize) -> Result<()> {
        match self {
            Self::Cbz {
                archive,
                file_names,
                pages,
                ..
            } => {
                let Some(file_name) = file_names.get(page - 1) else {
                    return Err(Error::PageNotFound(page));
                };
                let mut image = archive.raw_read_by_name(file_name.as_str())?;
                #[allow(clippy::cast_possible_truncation)]
                let mut bytes = Vec::with_capacity(image.size() as usize);
                std::io::copy(&mut image, &mut bytes)?;
                pages.push(base64::engine::general_purpose::STANDARD.encode(bytes));
            }
            Self::Epub { doc, pages, .. } => {
                doc.set_current_page(page - 1);
                let Some(content) = doc.get_current_with_epub_uris().ok() else {
                    return Err(Error::PageNotFound(page));
                };
                let content = String::from_utf8_lossy(&content);
                let mut dom = tl::parse(content.as_ref(), ParserOptions::default())?;
                try_for_each_tag_mut(&mut dom, "img", |tag| {
                    let Some(Some(src)) = tag.attributes_mut().get_mut("src") else {
                        debug!("attribute src not found in img tag {tag:?}");
                        return Ok(());
                    };
                    let Some(res) = doc.get_resource_by_path(&src.as_utf8_str().as_ref()[7..])
                    else {
                        return Ok(());
                    };
                    if let Some(bytes) = Some(base64::engine::general_purpose::STANDARD.encode(res))
                    {
                        *src = format!("data:image/png;base64,{bytes}").try_into()?;
                    }
                    Ok(())
                })?;
                pages.push(dom.outer_html());
            }
        }
        Ok(())
    }

    #[must_use]
    pub fn content_for_page(&self, page: usize) -> Option<String> {
        match self {
            Self::Cbz { pages, .. } | Self::Epub { pages, .. } => pages.get(page - 1).cloned(),
        }
    }

    #[must_use]
    pub fn max_page(&self) -> usize {
        match self {
            Self::Cbz { max_page, .. } | Self::Epub { max_page, .. } => *max_page,
        }
    }
}

fn try_for_each_tag_mut<F>(dom: &mut VDom, selector: &str, mut f: F) -> Result<()>
where
    F: FnMut(&mut HTMLTag<'_>) -> Result<()>,
{
    let Some(node_handles) = dom.query_selector(selector) else {
        debug!("no nodes found");
        return Ok(());
    };
    for node_handle in node_handles.collect::<Vec<_>>() {
        let Some(node) = node_handle.get_mut(dom.parser_mut()) else {
            debug!("node not found {}", node_handle.get_inner());
            continue;
        };
        let Some(tag) = node.as_tag_mut() else {
            debug!("node is not a tag {node:?}");
            continue;
        };
        f(tag)?;
    }

    Ok(())
}

#[allow(clippy::module_name_repetitions)]
pub type SharedDoc = Arc<Mutex<Doc>>;

/// ## Errors
pub fn try_load_shared_doc_from_path(
    type_: FileType,
    path: &Utf8Path,
) -> Result<(usize, SharedDoc)> {
    let doc = Doc::try_load_from_path(type_, path)?;

    Ok((doc.max_page(), Arc::new(Mutex::new(doc))))
}
