use std::{fs, io::BufReader};

use eco_cbz::image::ImageBytes;
use html5ever::{ParseOpts, parse_document, tendril::TendrilSink};
use markup5ever_rcdom::{Node, NodeData, RcDom};
use mobi::Mobi;
use tracing::{error, warn};

use crate::{Result, utils::base_32};

use super::MobiVersion;

#[allow(clippy::missing_errors_doc)]
pub fn convert_to_imgs(mobi: &Mobi) -> Result<Vec<ImageBytes<'_>>> {
    // Or is it `gen_version`? Both were equal in all the files I tested.
    let version = MobiVersion::try_from(mobi.metadata.mobi.format_version)?;
    let dom = get_dom(mobi)?;
    let imgs = mobi.image_records();
    let mut all_imgs = Vec::with_capacity(imgs.len());
    visit_node(version, &dom.document, |fid| {
        if let Some(img) = imgs.get(fid) {
            match img.content.try_into() {
                Ok(img) => all_imgs.push(img),
                Err(err) => error!("failed to decode image: {err}"),
            }
        } else {
            warn!("unknown fid {fid}");
        }
    });
    Ok(all_imgs)
}

fn get_dom(mobi: &Mobi) -> Result<RcDom> {
    let html = mobi.content_as_string_lossy();
    fs::write("index.html", html.as_bytes())?;
    let mut buf = BufReader::new(html.as_bytes());
    let dom = parse_document(RcDom::default(), ParseOpts::default())
        .from_utf8()
        .read_from(&mut buf)?;
    Ok(dom)
}

fn visit_node<F>(version: MobiVersion, node: &Node, mut f: F)
where
    F: FnMut(usize),
{
    visit_node_impl(version, node, &mut f);
}

fn visit_node_impl<F>(version: MobiVersion, node: &Node, f: &mut F)
where
    F: FnMut(usize),
{
    for node in node.children.borrow().iter() {
        if let NodeData::Element { name, attrs, .. } = &node.data {
            if name.local.as_ref() == "img" {
                for attr in attrs.borrow().iter() {
                    if version == MobiVersion::Mobi6 && attr.name.local.as_ref() == "recindex" {
                        let recindex: &str = attr.value.as_ref();
                        let fid = String::from_utf8_lossy(recindex.as_bytes())
                            .parse()
                            .unwrap();
                        f(fid);
                        continue;
                    }
                    if version == MobiVersion::Mobi8 && attr.name.local.as_ref() == "src" {
                        let src: &str = attr.value.as_ref();
                        // Encoding may be broken so we use a "best effort" strategy
                        // instead of simply extracting the fid and mime type from the string
                        let Some(index) = src.find("?mime=") else {
                            warn!("mime type not found for {src}");
                            continue;
                        };
                        // We assume the code is running on a 64bit system, so it's safe to unwrap
                        let fid = usize::try_from(base_32(&src.as_bytes()[index - 4..index]))
                            .unwrap()
                            - 1;
                        f(fid);
                    }
                }
            }
        }
        visit_node_impl(version, node, f);
    }
}
