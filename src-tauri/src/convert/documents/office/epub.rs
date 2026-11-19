//! Conversión hacia y desde EPUB (.epub).

use quick_xml::Reader;
use quick_xml::events::Event;

use super::super::{Block, blocks_to_html_body, markdown_to_blocks, markup};
use super::zip_utils::{
    attribute, build_zip_with_mimetype, find_attribute_in_xml, read_zip_entry, xml_escape,
};

pub fn read_epub(bytes: &[u8]) -> Result<Vec<Block>, String> {
    let container = read_zip_entry(bytes, "META-INF/container.xml", "EPUB")?;
    let container = String::from_utf8_lossy(&container);
    let rootfile = find_attribute_in_xml(&container, "rootfile", "full-path")
        .ok_or_else(|| "El EPUB no declara un archivo de contenido.".to_string())?;
    let opf = read_zip_entry(bytes, &rootfile, "EPUB")?;
    let opf = String::from_utf8_lossy(&opf);
    let base = rootfile
        .rsplit_once('/')
        .map(|(folder, _)| format!("{folder}/"))
        .unwrap_or_default();

    // Manifiesto id -> href
    let mut manifest = std::collections::HashMap::new();
    let mut reader = Reader::from_str(&opf);
    while let Ok(event) = reader.read_event() {
        if matches!(event, Event::Eof) {
            break;
        }
        if let Event::Start(start) | Event::Empty(start) = event
            && start.local_name().as_ref() == "item"
            && let (Some(id), Some(href)) = (attribute(&start, "id"), attribute(&start, "href"))
        {
            manifest.insert(id, href);
        }
    }
    // Orden del spine
    let mut reader = Reader::from_str(&opf);
    let mut order = Vec::new();
    while let Ok(event) = reader.read_event() {
        if matches!(event, Event::Eof) {
            break;
        }
        if let Event::Start(start) | Event::Empty(start) = event
            && start.local_name().as_ref() == "itemref"
            && let Some(idref) = attribute(&start, "idref")
            && let Some(href) = manifest.get(&idref)
        {
            order.push(href.clone());
        }
    }
    if order.is_empty() {
        order = manifest.values().cloned().collect();
    }

    let mut blocks = Vec::new();
    for href in order {
        let path = format!("{base}{href}");
        let Ok(raw) = read_zip_entry(bytes, &path, "EPUB") else {
            continue;
        };
        let html = String::from_utf8_lossy(&raw);
        let markdown = markup::html_to_markdown(&html)?;
        blocks.extend(markdown_to_blocks(&markdown));
    }
    if blocks.is_empty() {
        return Err("El EPUB no contiene capítulos de texto legibles.".to_string());
    }
    Ok(blocks)
}

pub fn write_epub(blocks: &[Block], title: &str) -> Result<Vec<u8>, String> {
    let body = blocks_to_html_body(blocks);
    let safe_title = xml_escape(title);
    let content = format!(
        r#"<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE html>
<html xmlns="http://www.w3.org/1999/xhtml" xml:lang="es">
<head><meta charset="utf-8"/><title>{safe_title}</title>
<style>body{{font-family:serif;line-height:1.6;margin:1em}}h1,h2,h3{{line-height:1.25}}</style></head>
<body><h1>{safe_title}</h1>{body}</body></html>"#
    );
    let container = r#"<?xml version="1.0" encoding="UTF-8"?>
<container version="1.0" xmlns="urn:oasis:names:tc:opendocument:xmlns:container">
<rootfiles><rootfile full-path="OEBPS/content.opf" media-type="application/oebps-package+xml"/></rootfiles>
</container>"#;
    let opf = format!(
        r#"<?xml version="1.0" encoding="UTF-8"?>
<package xmlns="http://www.idpf.org/2007/opf" version="2.0" unique-identifier="bookid">
<metadata xmlns:dc="http://purl.org/dc/elements/1.1/">
<dc:title>{safe_title}</dc:title><dc:identifier id="bookid">urn:uuid:fileflow-{safe_title}</dc:identifier>
<dc:language>es</dc:language><dc:creator>FileFlow</dc:creator>
</metadata>
<manifest><item id="content" href="content.xhtml" media-type="application/xhtml+xml"/><item id="ncx" href="toc.ncx" media-type="application/x-dtbncx+xml"/></manifest>
<spine toc="ncx"><itemref idref="content"/></spine>
</package>"#
    );
    let ncx = format!(
        r#"<?xml version="1.0" encoding="UTF-8"?>
<ncx xmlns="http://www.daisy.org/z3986/2005/ncx/" version="2005-1">
<head><meta name="dtb:uid" content="urn:uuid:fileflow-{safe_title}"/></head>
<docTitle><text>{safe_title}</text></docTitle>
<navMap><navPoint id="nav1" playOrder="1"><navLabel><text>{safe_title}</text></navLabel><content src="content.xhtml"/></navPoint></navMap>
</ncx>"#
    );
    build_zip_with_mimetype(
        "application/epub+zip",
        &[
            ("META-INF/container.xml", container.as_bytes().to_vec()),
            ("OEBPS/content.opf", opf.into_bytes()),
            ("OEBPS/content.xhtml", content.into_bytes()),
            ("OEBPS/toc.ncx", ncx.into_bytes()),
        ],
    )
}
