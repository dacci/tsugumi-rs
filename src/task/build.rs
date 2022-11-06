use crate::model::{Book, Chapter, Orientation, Page, TitleType};
use anyhow::{anyhow, Context as _, Result};
use chrono::{SecondsFormat, Utc};
use indexmap::IndexMap as Map;
use std::fs::File;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::rc::Rc;
use tempfile::{NamedTempFile, TempPath};
use tracing::{debug, info, warn};
use xml::writer::XmlEvent;
use xml::{EmitterConfig, EventWriter};
use zip::write::FileOptions;
use zip::{CompressionMethod, ZipWriter};

#[derive(clap::Args)]
pub(super) struct Args {
    /// Output EPub file in PATH.
    #[arg(short, long, value_name = "PATH")]
    output: Option<PathBuf>,
}

pub(super) fn main(args: Args) -> Result<()> {
    let path = find_project()?;

    let cx = Builder::new(&path)?.build()?;

    let output = args
        .output
        .as_deref()
        .or_else(|| path.parent())
        .unwrap_or_else(|| Path::new(""));
    cx.write_to(output)
}

fn find_project() -> Result<PathBuf> {
    let start = std::env::current_dir().context("failed to get current directory")?;

    let mut current = start.as_path();
    loop {
        let path = current.join("tsugumi.yaml");
        if path.exists() {
            break Ok(path);
        }

        if let Some(parent) = current.parent() {
            current = parent;
        } else {
            break Err(anyhow!(
                "could not find `tsugumi.yaml` in `{}` or any parent directory",
                start.display()
            ));
        }
    }
}

struct Builder {
    root: PathBuf,
    book: Rc<Book>,
}

impl Builder {
    fn new(path: impl AsRef<Path>) -> Result<Self> {
        let path = path.as_ref();
        let file =
            File::open(path).with_context(|| format!("failed to open `{}`", path.display()))?;
        let book: Book = serde_yaml::from_reader(file)
            .with_context(|| format!("failed to read `{}`", path.display()))?;

        Ok(Self {
            root: path.parent().unwrap().to_path_buf(),
            book: Rc::new(book),
        })
    }

    fn build(&self) -> Result<Context> {
        let mut cx = Context {
            book: Rc::clone(&self.book),
            title: self
                .book
                .metadata
                .title
                .iter()
                .find(|t| t.title_type == TitleType::Main)
                .or_else(|| self.book.metadata.title.first())
                .map(|t| t.name.as_str().to_string())
                .unwrap_or_default(),
            ..Default::default()
        };

        if self.book.rendition.style.is_empty() {
            self.build_default_style(&mut cx)?;
        } else {
            self.build_style(&mut cx)?;
        }

        for chapter in &self.book.chapter {
            self.build_chapter(&mut cx, chapter)?;
        }

        Ok(cx)
    }

    fn build_default_style(&self, cx: &mut Context) -> Result<()> {
        info!("building default style");

        let mut file = NamedTempFile::new()?;
        file.write_all(include_bytes!("../default-style.css"))?;

        let item = Item {
            media_type: "text/css".to_string(),
            href: "style/default.css".to_string(),
            properties: None,
            src: file.into_temp_path().into(),
        };

        let id = "s-default".to_string();
        cx.manifest.insert(id.clone(), item);
        cx.styles.push(id);

        Ok(())
    }

    fn build_style(&self, cx: &mut Context) -> Result<()> {
        info!("building style");

        for (style, seq) in self.book.rendition.style.iter().zip(1..) {
            let mut file = NamedTempFile::new()?;
            file.write_all(style.src.as_bytes())?;
            let src = file.into_temp_path();

            let item = Item {
                media_type: "text/css".to_string(),
                href: format!("style/{}", style.href),
                properties: None,
                src: src.into(),
            };

            let id = format!("s-{seq:04}");
            cx.manifest.insert(id.clone(), item);

            if style.link {
                cx.styles.push(id);
            }
        }

        Ok(())
    }

    fn build_chapter(&self, cx: &mut Context, chapter: &Chapter) -> Result<()> {
        info!(
            "building chapter {}",
            chapter.name.as_deref().unwrap_or("(untitled)")
        );

        let mut first = true;
        for page in &chapter.page {
            let id = self.build_page(cx, chapter, page)?;
            if first {
                first = false;

                if let Some(name) = &chapter.name {
                    cx.toc.insert(id, name.clone());
                }
            }
        }

        Ok(())
    }

    fn build_page(&self, cx: &mut Context, chapter: &Chapter, page: &Page) -> Result<String> {
        debug!("building page from {}", page.src.display());

        let src = self.root.join(&page.src);

        let (width, height) = {
            let img =
                image::open(&src).with_context(|| format!("failed to read {}", src.display()))?;
            (img.width(), img.height())
        };

        match self.book.rendition.orientation {
            Orientation::Landscape if width < height => {
                warn!("`{}` is a portrait page", page.src.display())
            }
            Orientation::Portrait if height < width => {
                warn!("`{}` is a landscape page", page.src.display())
            }
            _ => {}
        }

        let id = cx.add_image(src.as_path(), chapter.cover);
        let image = cx.manifest.get(&id).unwrap();

        let mut file = NamedTempFile::new()?;

        writeln!(file, r#"<?xml version="1.0" encoding="utf-8"?>"#)?;
        writeln!(file, r#"<!DOCTYPE html>"#)?;

        let mut writer = EventWriter::new_with_config(
            file,
            EmitterConfig::new()
                .perform_indent(true)
                .write_document_declaration(false),
        );

        writer.write(
            XmlEvent::start_element("html")
                .default_ns("http://www.w3.org/1999/xhtml")
                .ns("epub", "http://www.idpf.org/2007/ops")
                .attr("xml:lang", &self.book.metadata.language),
        )?;

        writer.write(XmlEvent::start_element("head"))?;

        writer.write(XmlEvent::start_element("meta").attr("charset", "UTF-8"))?;
        writer.write(XmlEvent::end_element())?; // meta

        writer.write(XmlEvent::start_element("title"))?;
        writer.write(XmlEvent::characters(&cx.title))?;
        writer.write(XmlEvent::end_element())?; // title

        for id in &cx.styles {
            let item = cx.manifest.get(id).unwrap();
            writer.write(
                XmlEvent::start_element("link")
                    .attr("rel", "stylesheet")
                    .attr("type", item.media_type.as_str())
                    .attr("href", &format!("../{}", item.href)),
            )?;
            writer.write(XmlEvent::end_element())?; // link
        }

        writer.write(
            XmlEvent::start_element("meta")
                .attr("name", "viewport")
                .attr("content", &format!("width={width}, height={height}")),
        )?;
        writer.write(XmlEvent::end_element())?; // meta

        writer.write(XmlEvent::end_element())?; // head

        let mut event = XmlEvent::start_element("body");
        if chapter.cover {
            event = event.attr("epub:type", "cover");
        }
        writer.write(event)?;

        writer.write(XmlEvent::start_element("div").attr("class", "main"))?;

        writer.write(
            XmlEvent::start_element("svg")
                .default_ns("http://www.w3.org/2000/svg")
                .ns("xlink", "http://www.w3.org/1999/xlink")
                .attr("version", "1.1")
                .attr("width", "100%")
                .attr("height", "100%")
                .attr("viewBox", &format!("0 0 {width} {height}")),
        )?;
        writer.write(
            XmlEvent::start_element("image")
                .attr("width", &width.to_string())
                .attr("height", &height.to_string())
                .attr("xlink:href", &format!("../{}", image.href)),
        )?;

        writer.write(XmlEvent::end_element())?; // image
        writer.write(XmlEvent::end_element())?; // svg
        writer.write(XmlEvent::end_element())?; // div
        writer.write(XmlEvent::end_element())?; // body
        writer.write(XmlEvent::end_element())?; // html

        let id = cx.add_page(writer.into_inner().into_temp_path(), chapter.cover);

        let props = if chapter.cover {
            Some("rendition:page-spread-center".to_string())
        } else {
            None
        };
        cx.add_spine(id.clone(), props);

        Ok(id)
    }
}

struct Item {
    media_type: String,
    href: String,
    properties: Option<String>,
    src: Resource,
}

enum Resource {
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
            Self::PathBuf(path) => path.as_path(),
            Self::TempPath(path) => path.as_ref(),
        }
    }
}

#[derive(Default)]
struct ItemRef {
    id_ref: String,
    linear: bool,
    properties: Option<String>,
}

#[derive(Default)]
struct Context {
    book: Rc<Book>,
    title: String,
    manifest: Map<String, Item>,
    spine: Vec<ItemRef>,
    styles: Vec<String>,
    image_index: usize,
    page_index: usize,
    toc: Map<String, String>,
}

impl Context {
    fn add_image(&mut self, src: impl Into<Resource>, cover: bool) -> String {
        let src = src.into();
        let mime = mime_guess::from_path(&src).first_or_octet_stream();
        let ext = src
            .as_ref()
            .extension()
            .and_then(|e| e.to_str())
            .map(|e| format!(".{e}"))
            .unwrap_or_default();

        let (id, properties) = if cover {
            ("cover".to_string(), Some("cover-image".to_string()))
        } else {
            self.image_index += 1;
            (format!("i-{:04}", self.image_index), None)
        };

        let item = Item {
            media_type: mime.to_string(),
            href: format!("image/{id}{ext}"),
            properties,
            src,
        };

        self.manifest.insert(id.clone(), item);

        id
    }

    fn add_page(&mut self, src: impl Into<Resource>, cover: bool) -> String {
        let id = if cover {
            "p-cover".to_string()
        } else {
            self.page_index += 1;
            format!("p-{:04}", self.page_index)
        };

        let item = Item {
            media_type: "application/xhtml+xml".to_string(),
            href: format!("xhtml/{id}.xhtml"),
            properties: Some("svg".to_string()),
            src: src.into(),
        };

        self.manifest.insert(id.clone(), item);

        id
    }

    fn add_spine(&mut self, id_ref: String, properties: Option<String>) {
        self.spine.push(ItemRef {
            id_ref,
            linear: true,
            properties,
        })
    }

    fn write_to(&self, path: impl AsRef<Path>) -> Result<()> {
        let path = path.as_ref().join(format!("{}.epub", self.title));
        let file = File::create(path)?;
        let mut zip = ZipWriter::new(file);

        self.write_mimetype(&mut zip)?;
        self.write_container(&mut zip)?;
        self.write_package(&mut zip)?;
        self.write_navigation(&mut zip)?;

        info!("writing items");
        for (_, item) in &self.manifest {
            zip.start_file(format!("item/{}", item.href), FileOptions::default())?;
            let mut file = File::open(&item.src)?;
            std::io::copy(&mut file, &mut zip)?;
        }

        Ok(())
    }

    fn write_mimetype(&self, zip: &mut ZipWriter<File>) -> Result<()> {
        info!("writing mimetype");

        zip.start_file(
            "mimetype",
            FileOptions::default().compression_method(CompressionMethod::Stored),
        )?;

        zip.write_all(b"application/epub+zip")?;

        Ok(())
    }

    fn write_container(&self, zip: &mut ZipWriter<File>) -> Result<()> {
        info!("writing container");

        zip.start_file("META-INF/container.xml", FileOptions::default())?;
        let mut w = EventWriter::new_with_config(zip, EmitterConfig::new().perform_indent(true));

        w.write(
            XmlEvent::start_element("container")
                .default_ns("urn:oasis:names:tc:opendocument:xmlns:container")
                .attr("version", "1.0"),
        )?;

        w.write(XmlEvent::start_element("rootfiles"))?;

        w.write(
            XmlEvent::start_element("rootfile")
                .attr("full-path", "item/standard.opf")
                .attr("media-type", "application/oebps-package+xml"),
        )?;

        w.write(XmlEvent::end_element())?; // rootfile
        w.write(XmlEvent::end_element())?; // rootfiles
        w.write(XmlEvent::end_element())?; // container

        Ok(())
    }

    fn write_package(&self, zip: &mut ZipWriter<File>) -> Result<()> {
        info!("writing package");

        zip.start_file("item/standard.opf", FileOptions::default())?;
        let mut w = EventWriter::new_with_config(zip, EmitterConfig::new().perform_indent(true));

        w.write(
            XmlEvent::start_element("package")
                .default_ns("http://www.idpf.org/2007/opf")
                .attr("version", "3.0")
                .attr("xml:lang", &self.book.metadata.language)
                .attr("unique-identifier", "unique-id")
                .attr("prefix", "ebpaj: http://www.ebpaj.jp/"),
        )?;

        self.write_package_metadata(&mut w)?;
        self.write_package_manifest(&mut w)?;
        self.write_package_spine(&mut w)?;

        w.write(XmlEvent::end_element())?;

        Ok(())
    }

    fn write_package_metadata<W: Write>(&self, w: &mut EventWriter<W>) -> Result<()> {
        w.write(XmlEvent::start_element("metadata").ns("dc", "http://purl.org/dc/elements/1.1/"))?;

        for (title, seq) in self.book.metadata.title.iter().zip(1..) {
            let refines = format!("#title{seq}");

            w.write(XmlEvent::start_element("dc:title").attr("id", &refines[1..]))?;
            w.write(XmlEvent::characters(&title.name))?;
            w.write(XmlEvent::end_element())?;

            w.write(
                XmlEvent::start_element("meta")
                    .attr("refines", &refines)
                    .attr("property", "title-type"),
            )?;
            w.write(XmlEvent::characters(title.title_type.as_ref()))?;
            w.write(XmlEvent::end_element())?;

            if let Some(value) = &title.alternate_script {
                w.write(
                    XmlEvent::start_element("meta")
                        .attr("refines", &refines)
                        .attr("property", "alternate-script"),
                )?;
                w.write(XmlEvent::characters(value))?;
                w.write(XmlEvent::end_element())?;
            }

            if let Some(value) = &title.file_as {
                w.write(
                    XmlEvent::start_element("meta")
                        .attr("refines", &refines)
                        .attr("property", "file-as"),
                )?;
                w.write(XmlEvent::characters(value))?;
                w.write(XmlEvent::end_element())?;
            }

            w.write(
                XmlEvent::start_element("meta")
                    .attr("refines", &refines)
                    .attr("property", "display-seq"),
            )?;
            w.write(XmlEvent::characters(&seq.to_string()))?;
            w.write(XmlEvent::end_element())?;
        }

        for (creator, seq) in self.book.metadata.creator.iter().zip(1..) {
            let refines = format!("#creator{seq}");

            w.write(XmlEvent::start_element("dc:creator").attr("id", &refines[1..]))?;
            w.write(XmlEvent::characters(&creator.name))?;
            w.write(XmlEvent::end_element())?;

            if let Some(value) = &creator.role {
                w.write(
                    XmlEvent::start_element("meta")
                        .attr("refines", &refines)
                        .attr("property", "role")
                        .attr("scheme", "marc:relators"),
                )?;
                w.write(XmlEvent::characters(value))?;
                w.write(XmlEvent::end_element())?;
            }

            if let Some(value) = &creator.alternate_script {
                w.write(
                    XmlEvent::start_element("meta")
                        .attr("refines", &refines)
                        .attr("property", "alternate-script"),
                )?;
                w.write(XmlEvent::characters(value))?;
                w.write(XmlEvent::end_element())?;
            }

            if let Some(value) = &creator.file_as {
                w.write(
                    XmlEvent::start_element("meta")
                        .attr("refines", &refines)
                        .attr("property", "file-as"),
                )?;
                w.write(XmlEvent::characters(value))?;
                w.write(XmlEvent::end_element())?;
            }

            w.write(
                XmlEvent::start_element("meta")
                    .attr("refines", &refines)
                    .attr("property", "display-seq"),
            )?;
            w.write(XmlEvent::characters(&format!("{}", seq)))?;
            w.write(XmlEvent::end_element())?;
        }

        for (contributor, seq) in self.book.metadata.contributor.iter().zip(1..) {
            let refines = format!("#creator{seq}");

            w.write(XmlEvent::start_element("dc:creator").attr("id", &refines[1..]))?;
            w.write(XmlEvent::characters(&contributor.name))?;
            w.write(XmlEvent::end_element())?;

            if let Some(value) = &contributor.role {
                w.write(
                    XmlEvent::start_element("meta")
                        .attr("refines", &refines)
                        .attr("property", "role")
                        .attr("scheme", "marc:relators"),
                )?;
                w.write(XmlEvent::characters(value))?;
                w.write(XmlEvent::end_element())?;
            }

            if let Some(value) = &contributor.alternate_script {
                w.write(
                    XmlEvent::start_element("meta")
                        .attr("refines", &refines)
                        .attr("property", "alternate-script"),
                )?;
                w.write(XmlEvent::characters(value))?;
                w.write(XmlEvent::end_element())?;
            }

            if let Some(value) = &contributor.file_as {
                w.write(
                    XmlEvent::start_element("meta")
                        .attr("refines", &refines)
                        .attr("property", "file-as"),
                )?;
                w.write(XmlEvent::characters(value))?;
                w.write(XmlEvent::end_element())?;
            }

            w.write(
                XmlEvent::start_element("meta")
                    .attr("refines", &refines)
                    .attr("property", "display-seq"),
            )?;
            w.write(XmlEvent::characters(&format!("{}", seq)))?;
            w.write(XmlEvent::end_element())?;
        }

        for (collection, seq) in self.book.metadata.collection.iter().zip(1..) {
            let refines = format!("#collection{seq}");

            w.write(
                XmlEvent::start_element("meta")
                    .attr("property", "belongs-to-collection")
                    .attr("id", &refines[1..]),
            )?;
            w.write(XmlEvent::characters(&collection.name))?;
            w.write(XmlEvent::end_element())?;

            w.write(
                XmlEvent::start_element("meta")
                    .attr("refines", &refines)
                    .attr("property", "collection-type"),
            )?;
            w.write(XmlEvent::characters(collection.collection_type.as_ref()))?;
            w.write(XmlEvent::end_element())?;

            if let Some(value) = collection.position {
                w.write(
                    XmlEvent::start_element("meta")
                        .attr("refines", &refines)
                        .attr("property", "group-position"),
                )?;
                w.write(XmlEvent::characters(&value.to_string()))?;
                w.write(XmlEvent::end_element())?;
            }
        }

        w.write(XmlEvent::start_element("dc:language"))?;
        w.write(XmlEvent::characters(&self.book.metadata.language))?;
        w.write(XmlEvent::end_element())?;

        w.write(XmlEvent::start_element("dc:identifier").attr("id", "unique-id"))?;
        w.write(XmlEvent::characters(&self.book.metadata.identifier))?;
        w.write(XmlEvent::end_element())?;

        w.write(XmlEvent::start_element("meta").attr("property", "dcterms:modified"))?;
        w.write(XmlEvent::characters(
            &Utc::now().to_rfc3339_opts(SecondsFormat::Secs, true),
        ))?;
        w.write(XmlEvent::end_element())?;

        w.write(XmlEvent::start_element("meta").attr("property", "rendition:layout"))?;
        w.write(XmlEvent::characters(self.book.rendition.layout.as_ref()))?;
        w.write(XmlEvent::end_element())?;

        w.write(XmlEvent::start_element("meta").attr("property", "rendition:orientation"))?;
        w.write(XmlEvent::characters(
            self.book.rendition.orientation.as_ref(),
        ))?;
        w.write(XmlEvent::end_element())?;

        w.write(XmlEvent::start_element("meta").attr("property", "rendition:spread"))?;
        w.write(XmlEvent::characters(self.book.rendition.spread.as_ref()))?;
        w.write(XmlEvent::end_element())?;

        w.write(XmlEvent::start_element("meta").attr("property", "ebpaj:guide-version"))?;
        w.write(XmlEvent::characters("1.1.3"))?;
        w.write(XmlEvent::end_element())?;

        w.write(XmlEvent::end_element())?;

        Ok(())
    }

    fn write_package_manifest<W: Write>(&self, w: &mut EventWriter<W>) -> Result<()> {
        w.write(XmlEvent::start_element("manifest"))?;

        w.write(
            XmlEvent::start_element("item")
                .attr("media-type", "application/xhtml+xml")
                .attr("id", "toc")
                .attr("href", "navigation-documents.xhtml")
                .attr("properties", "nav"),
        )?;
        w.write(XmlEvent::end_element())?;

        for (id, item) in &self.manifest {
            let mut event = XmlEvent::start_element("item")
                .attr("media-type", &item.media_type)
                .attr("id", id)
                .attr("href", &item.href);
            if let Some(properties) = &item.properties {
                event = event.attr("properties", properties);
            }

            w.write(event)?;
            w.write(XmlEvent::end_element())?;
        }

        w.write(XmlEvent::end_element())?;

        Ok(())
    }

    fn write_package_spine<W: Write>(&self, w: &mut EventWriter<W>) -> Result<()> {
        w.write(XmlEvent::start_element("spine").attr(
            "page-progression-direction",
            self.book.rendition.direction.as_ref(),
        ))?;

        for item_ref in &self.spine {
            let mut event = XmlEvent::start_element("itemref")
                .attr("linear", if item_ref.linear { "yes" } else { "no" })
                .attr("idref", &item_ref.id_ref);
            if let Some(properties) = &item_ref.properties {
                event = event.attr("properties", properties);
            }

            w.write(event)?;
            w.write(XmlEvent::end_element())?;
        }

        w.write(XmlEvent::end_element())?;

        Ok(())
    }

    fn write_navigation(&self, zip: &mut ZipWriter<File>) -> Result<()> {
        info!("writing navigation");

        zip.start_file("item/navigation-documents.xhtml", FileOptions::default())?;

        writeln!(zip, r#"<?xml version="1.0" encoding="utf-8"?>"#)?;
        writeln!(zip, r#"<!DOCTYPE html>"#)?;

        let mut w = EventWriter::new_with_config(
            zip,
            EmitterConfig::new()
                .perform_indent(true)
                .write_document_declaration(false),
        );

        w.write(
            XmlEvent::start_element("html")
                .default_ns("http://www.w3.org/1999/xhtml")
                .ns("epub", "http://www.idpf.org/2007/ops")
                .attr("xml:lang", &self.book.metadata.language),
        )?;

        w.write(XmlEvent::start_element("head"))?;

        w.write(XmlEvent::start_element("meta").attr("charset", "UTF-8"))?;
        w.write(XmlEvent::end_element())?; // meta

        w.write(XmlEvent::start_element("title"))?;
        w.write(XmlEvent::characters("Navigation"))?;
        w.write(XmlEvent::end_element())?; // title

        w.write(XmlEvent::end_element())?; // head

        w.write(XmlEvent::start_element("body"))?;
        w.write(
            XmlEvent::start_element("nav")
                .attr("epub:type", "toc")
                .attr("id", "toc"),
        )?;

        w.write(XmlEvent::start_element("h1"))?;
        w.write(XmlEvent::characters("Navigation"))?;
        w.write(XmlEvent::end_element())?; // h1

        w.write(XmlEvent::start_element("ol"))?;

        for (id, title) in &self.toc {
            let item = self.manifest.get(id).unwrap();

            w.write(XmlEvent::start_element("li"))?;
            w.write(XmlEvent::start_element("a").attr("href", &item.href))?;
            w.write(XmlEvent::characters(title))?;
            w.write(XmlEvent::end_element())?; // a
            w.write(XmlEvent::end_element())?; // li
        }

        w.write(XmlEvent::end_element())?; // ol
        w.write(XmlEvent::end_element())?; // nav
        w.write(XmlEvent::end_element())?; // body
        w.write(XmlEvent::end_element())?; // html

        Ok(())
    }
}
