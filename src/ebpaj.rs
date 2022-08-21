use crate::Direction;
use chrono::{SecondsFormat, Utc};
use indexmap::IndexMap;
use std::collections::BTreeMap as Map;
use std::fmt;
use std::fs::File;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::rc::Rc;
use tempfile::TempPath;
use uuid::Uuid;
use xml::writer::XmlEvent;
use xml::{EmitterConfig, EventWriter};
use zip::write::{FileOptions, ZipWriter};
use zip::CompressionMethod;

pub enum Resource {
    PathBuf(PathBuf),
    TempPath(TempPath),
}

impl From<&Path> for Resource {
    fn from(path: &Path) -> Self {
        Self::PathBuf(path.to_path_buf())
    }
}

impl From<PathBuf> for Resource {
    fn from(path: PathBuf) -> Self {
        Self::PathBuf(path)
    }
}

impl From<TempPath> for Resource {
    fn from(path: TempPath) -> Self {
        Self::TempPath(path)
    }
}

impl AsRef<Path> for Resource {
    fn as_ref(&self) -> &Path {
        match self {
            Resource::PathBuf(path) => path.as_path(),
            Resource::TempPath(path) => path.as_ref(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Href(&'static str, String);

impl fmt::Display for Href {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}/{}", self.0, self.1)
    }
}

pub struct Item {
    pub media_type: String,
    pub href: Href,
    pub props: Option<String>,
    pub path: Resource,
}

pub struct ItemRef {
    linear: bool,
    idref: String,
    props: String,
}

#[derive(Default)]
pub struct Builder {
    title: Option<String>,
    subtitle: Option<String>,
    author: Option<String>,
    series_title: Option<String>,
    series_position: Option<String>,
    set_title: Option<String>,
    set_position: Option<String>,
    dir: Direction,
    items: Map<String, Rc<Item>>,
    spine: Vec<ItemRef>,
    nav: IndexMap<String, Href>,
}

impl Builder {
    const STYLE: &'static str = "style";
    const IMAGE: &'static str = "image";
    const XHTML: &'static str = "xhtml";

    pub fn new() -> Self {
        Default::default()
    }

    pub fn set_title(&mut self, title: &str) {
        self.title = Some(title.to_string());
    }

    pub fn title(mut self, title: &str) -> Self {
        self.set_title(title);
        self
    }

    pub fn set_subtitle(&mut self, subtitle: &str) {
        self.subtitle = Some(subtitle.to_string());
    }

    pub fn subtitle(mut self, subtitle: &str) -> Self {
        self.set_subtitle(subtitle);
        self
    }

    pub fn set_author(&mut self, author: &str) {
        self.author = Some(author.to_string());
    }

    pub fn author(mut self, author: &str) -> Self {
        self.set_author(author);
        self
    }

    pub fn set_series(&mut self, title: &str, position: Option<&str>) {
        self.series_title = Some(title.to_string());
        self.series_position = position.map(|s| s.to_string());
    }

    pub fn series(mut self, title: &str, position: Option<&str>) -> Self {
        self.set_series(title, position);
        self
    }

    pub fn set_set(&mut self, title: &str, position: Option<&str>) {
        self.set_title = Some(title.to_string());
        self.set_position = position.map(|s| s.to_string());
    }

    pub fn set(mut self, title: &str, position: Option<&str>) -> Self {
        self.set_set(title, position);
        self
    }

    pub fn set_direction(&mut self, dir: Direction) {
        self.dir = dir;
    }

    pub fn direction(mut self, dir: Direction) -> Self {
        self.set_direction(dir);
        self
    }

    pub fn add_style(&mut self, path: PathBuf, id: String) -> Rc<Item> {
        let item = Rc::new(Item {
            media_type: "text/css".to_string(),
            href: Href(
                Self::STYLE,
                path.file_name().unwrap().to_str().unwrap().to_string(),
            ),
            props: None,
            path: path.into(),
        });
        self.items.insert(id, Rc::clone(&item));

        item
    }

    pub fn add_image(&mut self, path: impl AsRef<Path>, props: Option<&str>) -> Rc<Item> {
        let path = path.as_ref();
        let media_type = mime_guess::from_path(path)
            .first_or_octet_stream()
            .to_string();

        let (id, href) = match props {
            Some("cover-image") => (
                "cover".to_string(),
                format!("cover.{}", path.extension().unwrap().to_str().unwrap()),
            ),
            _ => (
                format!("i-{}", path.file_stem().unwrap().to_str().unwrap()),
                path.file_name().unwrap().to_str().unwrap().to_string(),
            ),
        };

        let item = Rc::new(Item {
            media_type,
            href: Href(Self::IMAGE, href),
            props: props.map(ToOwned::to_owned),
            path: path.into(),
        });
        self.items.insert(id, Rc::clone(&item));

        item
    }

    pub fn add_xhtml(
        &mut self,
        path: impl Into<Resource>,
        id: &str,
        props: Option<&str>,
    ) -> Rc<Item> {
        let item = Rc::new(Item {
            media_type: "application/xhtml+xml".to_string(),
            href: Href(Self::XHTML, format!("{}.xhtml", id)),
            props: props.map(ToOwned::to_owned),
            path: path.into(),
        });
        self.items.insert(id.to_string(), Rc::clone(&item));

        item
    }

    pub fn add_page(&mut self, idref: &str, props: &str) {
        self.spine.push(ItemRef {
            linear: true,
            idref: idref.to_string(),
            props: props.to_string(),
        });
    }

    pub fn add_navigation(&mut self, caption: &str, href: &Href) {
        self.nav.insert(caption.to_string(), href.clone());
    }

    pub fn build(&self, path: &Path) -> anyhow::Result<()> {
        let mut zip = ZipWriter::new(File::create(path)?);

        zip.start_file(
            "mimetype",
            FileOptions::default().compression_method(CompressionMethod::Stored),
        )?;
        zip.write_all(b"application/epub+zip")?;

        zip.start_file("META-INF/container.xml", FileOptions::default())?;
        self.build_container(&mut zip)?;

        zip.start_file("item/navigation-documents.xhtml", FileOptions::default())?;
        self.build_navigation(&mut zip)?;

        for item in self.items.values() {
            let mut file = File::open(&item.path)?;
            zip.start_file(format!("item/{}", item.href), FileOptions::default())?;
            std::io::copy(&mut file, &mut zip)?;
        }

        zip.start_file("item/standard.opf", FileOptions::default())?;
        self.build_package(&mut zip)?;

        Ok(())
    }

    fn build_container<W: Write>(&self, w: &mut W) -> anyhow::Result<()> {
        let mut writer = EventWriter::new_with_config(w, EmitterConfig::new().perform_indent(true));

        writer.write(
            XmlEvent::start_element("container")
                .default_ns("urn:oasis:names:tc:opendocument:xmlns:container")
                .attr("version", "1.0"),
        )?;
        writer.write(XmlEvent::start_element("rootfiles"))?;
        writer.write(
            XmlEvent::start_element("rootfile")
                .attr("full-path", "item/standard.opf")
                .attr("media-type", "application/oebps-package+xml"),
        )?;

        writer.write(XmlEvent::end_element())?; // rootfile
        writer.write(XmlEvent::end_element())?; // rootfiles
        writer.write(XmlEvent::end_element())?; // container

        Ok(())
    }

    fn build_navigation<W: Write>(&self, w: &mut W) -> anyhow::Result<()> {
        let mut writer = EventWriter::new_with_config(w, EmitterConfig::new().perform_indent(true));

        writer.write(
            XmlEvent::start_element("html")
                .default_ns("http://www.w3.org/1999/xhtml")
                .ns("epub", "http://www.idpf.org/2007/ops")
                .attr("xml:lang", "ja"),
        )?;

        writer.write(XmlEvent::start_element("head"))?;

        writer.write(XmlEvent::start_element("meta").attr("charset", "UTF-8"))?;
        writer.write(XmlEvent::end_element())?;

        writer.write(XmlEvent::start_element("title"))?;
        writer.write(XmlEvent::characters("Navigation"))?;
        writer.write(XmlEvent::end_element())?;

        writer.write(XmlEvent::end_element())?; // head

        writer.write(XmlEvent::start_element("body"))?;

        writer.write(
            XmlEvent::start_element("nav")
                .attr("epub:type", "toc")
                .attr("id", "toc"),
        )?;

        writer.write(XmlEvent::start_element("h1"))?;
        writer.write(XmlEvent::characters("Navigation"))?;
        writer.write(XmlEvent::end_element())?;

        writer.write(XmlEvent::start_element("ol"))?;

        for (caption, href) in &self.nav {
            writer.write(XmlEvent::start_element("li"))?;
            writer.write(XmlEvent::start_element("a").attr("href", &href.to_string()))?;
            writer.write(XmlEvent::characters(caption))?;
            writer.write(XmlEvent::end_element())?; // a
            writer.write(XmlEvent::end_element())?; // li
        }

        writer.write(XmlEvent::end_element())?; // ol
        writer.write(XmlEvent::end_element())?; // nav
        writer.write(XmlEvent::end_element())?; // body
        writer.write(XmlEvent::end_element())?; // html

        Ok(())
    }

    fn build_package<W: Write>(&self, w: &mut W) -> anyhow::Result<()> {
        let mut writer = EventWriter::new_with_config(w, EmitterConfig::new().perform_indent(true));

        writer.write(
            XmlEvent::start_element("package")
                .default_ns("http://www.idpf.org/2007/opf")
                .attr("version", "3.0")
                .attr("unique-identifier", "unique-id")
                .attr("xml:lang", "ja")
                .attr("dir", self.dir.as_ref())
                .attr("prefix", "ebpaj: http://www.ebpaj.jp/"),
        )?;

        writer.write(
            XmlEvent::start_element("metadata").ns("dc", "http://purl.org/dc/elements/1.1/"),
        )?;

        let mut digest = md5::Context::new();

        if let Some(title) = &self.title {
            writer.write(XmlEvent::start_element("dc:title").attr("id", "title"))?;
            writer.write(XmlEvent::characters(title))?;
            writer.write(XmlEvent::end_element())?;

            writer.write(
                XmlEvent::start_element("meta")
                    .attr("refines", "#title")
                    .attr("property", "title-type"),
            )?;
            writer.write(XmlEvent::characters("main"))?;
            writer.write(XmlEvent::end_element())?;

            digest.write_all(title.as_bytes())?;
        }

        if let Some(subtitle) = &self.subtitle {
            writer.write(XmlEvent::start_element("dc:title").attr("id", "subtitle"))?;
            writer.write(XmlEvent::characters(subtitle))?;
            writer.write(XmlEvent::end_element())?;

            writer.write(
                XmlEvent::start_element("meta")
                    .attr("refines", "#subtitle")
                    .attr("property", "title-type"),
            )?;
            writer.write(XmlEvent::characters("subtitle"))?;
            writer.write(XmlEvent::end_element())?;

            digest.write_all(&[0])?;
            digest.write_all(subtitle.as_bytes())?;
        }

        if let Some(author) = &self.author {
            writer.write(XmlEvent::start_element("dc:creator").attr("id", "author"))?;
            writer.write(XmlEvent::characters(author))?;
            writer.write(XmlEvent::end_element())?;

            writer.write(
                XmlEvent::start_element("meta")
                    .attr("refines", "#author")
                    .attr("property", "role")
                    .attr("scheme", "marc:relators"),
            )?;
            writer.write(XmlEvent::characters("aut"))?;
            writer.write(XmlEvent::end_element())?;

            digest.write_all(&[0])?;
            digest.write_all(author.as_bytes())?;
        }

        if let Some(title) = &self.series_title {
            writer.write(
                XmlEvent::start_element("meta")
                    .attr("id", "series")
                    .attr("property", "belongs-to-collection"),
            )?;
            writer.write(XmlEvent::characters(title))?;
            writer.write(XmlEvent::end_element())?;

            writer.write(
                XmlEvent::start_element("meta")
                    .attr("refines", "#series")
                    .attr("property", "collection-type"),
            )?;
            writer.write(XmlEvent::characters("series"))?;
            writer.write(XmlEvent::end_element())?;

            writer.write(
                XmlEvent::start_element("meta")
                    .attr("name", "calibre:series")
                    .attr("content", title),
            )?;
            writer.write(XmlEvent::end_element())?;

            digest.write_all(&[0])?;
            digest.write_all(title.as_bytes())?;

            if let Some(position) = &self.series_position {
                writer.write(
                    XmlEvent::start_element("meta")
                        .attr("refines", "#series")
                        .attr("property", "group-position"),
                )?;
                writer.write(XmlEvent::characters(position))?;
                writer.write(XmlEvent::end_element())?;

                writer.write(
                    XmlEvent::start_element("meta")
                        .attr("name", "calibre:series_index")
                        .attr("content", position),
                )?;
                writer.write(XmlEvent::end_element())?;

                digest.write_all(&[0])?;
                digest.write_all(position.as_bytes())?;
            }
        }

        if let Some(title) = &self.set_title {
            writer.write(
                XmlEvent::start_element("meta")
                    .attr("id", "set")
                    .attr("property", "belongs-to-collection"),
            )?;
            writer.write(XmlEvent::characters(title))?;
            writer.write(XmlEvent::end_element())?;

            writer.write(
                XmlEvent::start_element("meta")
                    .attr("refines", "#set")
                    .attr("property", "collection-type"),
            )?;
            writer.write(XmlEvent::characters("set"))?;
            writer.write(XmlEvent::end_element())?;

            digest.write_all(&[0])?;
            digest.write_all(title.as_bytes())?;

            if let Some(position) = &self.set_position {
                writer.write(
                    XmlEvent::start_element("meta")
                        .attr("refines", "#set")
                        .attr("property", "group-position"),
                )?;
                writer.write(XmlEvent::characters(position))?;
                writer.write(XmlEvent::end_element())?;

                digest.write_all(&[0])?;
                digest.write_all(position.as_bytes())?;
            }
        }

        writer.write(XmlEvent::start_element("dc:language"))?;
        writer.write(XmlEvent::characters("ja"))?;
        writer.write(XmlEvent::end_element())?;

        let uuid = {
            let mut digest: [u8; 16] = digest.compute().into();
            // Copied from java.util.UUID#nameUUIDFromBytes(byte[])
            digest[6] &= 0x0f; // clear version
            digest[6] |= 0x30; // set to version 3
            digest[8] &= 0x3f; // clear variant
            digest[8] |= 0x80; // set to IETF variant
            Uuid::from_slice(&digest)?
        };

        writer.write(XmlEvent::start_element("dc:identifier").attr("id", "unique-id"))?;
        writer.write(XmlEvent::characters(&format!("urn:uuid:{}", uuid)))?;
        writer.write(XmlEvent::end_element())?;

        writer.write(XmlEvent::start_element("meta").attr("property", "dcterms:modified"))?;
        writer.write(XmlEvent::characters(
            &Utc::now().to_rfc3339_opts(SecondsFormat::Secs, true),
        ))?;
        writer.write(XmlEvent::end_element())?;

        writer.write(XmlEvent::start_element("meta").attr("property", "rendition:layout"))?;
        writer.write(XmlEvent::characters("pre-paginated"))?;
        writer.write(XmlEvent::end_element())?;

        writer.write(XmlEvent::start_element("meta").attr("property", "rendition:spread"))?;
        writer.write(XmlEvent::characters("landscape"))?;
        writer.write(XmlEvent::end_element())?;

        writer.write(XmlEvent::start_element("meta").attr("property", "ebpaj:guide-version"))?;
        writer.write(XmlEvent::characters("1.1.3"))?;
        writer.write(XmlEvent::end_element())?;

        writer.write(XmlEvent::end_element())?; // metadata

        writer.write(XmlEvent::start_element("manifest"))?;

        writer.write(
            XmlEvent::start_element("item")
                .attr("media-type", "application/xhtml+xml")
                .attr("id", "toc")
                .attr("href", "navigation-documents.xhtml")
                .attr("properties", "nav"),
        )?;
        writer.write(XmlEvent::end_element())?;

        for (id, item) in &self.items {
            let href = item.href.to_string();

            let mut event = XmlEvent::start_element("item")
                .attr("media-type", &item.media_type)
                .attr("id", id)
                .attr("href", &href);
            if let Some(props) = &item.props {
                event = event.attr("properties", props)
            }

            writer.write(event)?;
            writer.write(XmlEvent::end_element())?;
        }

        writer.write(XmlEvent::end_element())?; // manifest

        writer.write(
            XmlEvent::start_element("spine").attr("page-progression-direction", self.dir.as_ref()),
        )?;

        for item in &self.spine {
            let linear = match item.linear {
                true => "yes",
                false => "no",
            };

            writer.write(
                XmlEvent::start_element("itemref")
                    .attr("linear", linear)
                    .attr("idref", &item.idref)
                    .attr("properties", &item.props),
            )?;
            writer.write(XmlEvent::end_element())?;
        }

        writer.write(XmlEvent::end_element())?; // spine

        writer.write(XmlEvent::end_element())?; // package

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_builder_add_style() {
        let mut builder = Builder::new();
        builder.add_style("test.css".into(), "test".into());

        let item = builder.items.get("test").unwrap();
        assert_eq!(item.media_type, "text/css");
        assert_eq!(item.href.0, "style");
        assert_eq!(item.href.1, "test.css");
    }

    #[test]
    fn test_builder_add_image() {
        let mut builder = Builder::new();
        builder.add_image("test.png", None);

        let item = builder.items.get("i-test").unwrap();
        assert_eq!(item.media_type, "image/png");
        assert_eq!(item.href.0, "image");
        assert_eq!(item.href.1, "test.png");
    }

    #[test]
    fn test_builder_add_cover_image() {
        let mut builder = Builder::new();
        builder.add_image("test.jpg", Some("cover-image"));

        let item = builder.items.get("cover").unwrap();
        assert_eq!(item.media_type, "image/jpeg");
        assert_eq!(item.href.0, "image");
        assert_eq!(item.href.1, "cover.jpg");
    }
}
