use clap::Parser;
use std::fs::File;
use std::path::{Path, PathBuf};
use tempfile::{NamedTempFile, TempPath};
use tsugumi::ebpaj::{Builder, Direction, Item};
use tsugumi::{Book, Style};
use xml::writer::XmlEvent;
use xml::{EmitterConfig, EventWriter};

#[derive(Parser)]
struct Args {
    #[clap(short, long)]
    style: PathBuf,
    #[clap(short, long)]
    output: Option<PathBuf>,
    input: PathBuf,
}

fn main() -> anyhow::Result<()> {
    let args: Args = Args::parse();

    let book = serde_yaml::from_reader(File::open(&args.input)?)?;
    let style = serde_yaml::from_reader(File::open(&args.style)?)?;

    let book_base = args
        .input
        .parent()
        .map(ToOwned::to_owned)
        .unwrap_or_default();
    let style_base = args
        .style
        .parent()
        .map(ToOwned::to_owned)
        .unwrap_or_default();

    let mut ctx = Context {
        book,
        book_base,
        style,
        style_base,
        styles: vec![],
    };
    let builder = ctx.build()?;

    let output = args.output.as_deref().unwrap_or_else(|| Path::new(""));
    let output = output.join(format!(
        "{} - {}.epub",
        ctx.book.metadata.title, ctx.book.metadata.author
    ));
    builder.build(&output)?;

    Ok(())
}

struct Context {
    book: Book,
    book_base: PathBuf,
    style: Style,
    style_base: PathBuf,
    styles: Vec<String>,
}

impl Context {
    fn build(&mut self) -> anyhow::Result<Builder> {
        let mut builder = Builder::new()
            .title(&self.book.metadata.title)
            .author(&self.book.metadata.author)
            .direction(Direction::RightToLeft);

        self.build_style(&mut builder);

        {
            let image =
                builder.add_image(self.book_base.join(&self.book.cover), Some("cover-image"));
            let xhtml = self.build_page(image, true)?;
            let page = builder
                .add_xhtml(xhtml, "p-cover", Some("svg"))
                .href
                .clone();
            builder.add_page("p-cover", "rendition:page-spread-center");
            builder.add_navigation("表紙", page);
        }

        for chapter in &self.book.chapters {
            for i in 0..chapter.pages.len() {
                let path = &chapter.pages[i];
                let image = builder.add_image(self.book_base.join(path), None);

                let id = format!("p-{}", path.file_stem().unwrap().to_str().unwrap());
                let path = self.build_page(image, false)?;
                let page = builder.add_xhtml(path, &id, Some("svg")).href.clone();

                let props = match i % 2 {
                    0 => "page-spread-left",
                    1 => "page-spread-right",
                    _ => unreachable!(),
                };
                builder.add_page(&id, props);

                if i == 0 && chapter.name.is_some() {
                    builder.add_navigation(chapter.name.as_ref().unwrap(), page);
                }
            }
        }

        Ok(builder)
    }

    fn build_style(&mut self, builder: &mut Builder) {
        for href in &self.style.links {
            let path = self.style_base.join(href);
            let id = href.to_id();
            let item = builder.add_style(path, id);
            self.styles.push(item.href.to_string())
        }

        for href in &self.style.includes {
            let path = self.style_base.join(href);
            let id = href.to_id();
            builder.add_style(path, id);
        }
    }

    fn build_page(&self, item: &Item, cover: bool) -> anyhow::Result<TempPath> {
        let (width, height) = {
            let img = image::open(&item.path)?;
            (img.width().to_string(), img.height().to_string())
        };

        let mut file = NamedTempFile::new()?;
        let mut writer = EventWriter::new_with_config(
            file.as_file_mut(),
            EmitterConfig::new().perform_indent(true),
        );

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
        writer.write(XmlEvent::characters(&self.book.metadata.title))?;
        writer.write(XmlEvent::end_element())?;

        for href in &self.styles {
            writer.write(
                XmlEvent::start_element("link")
                    .attr("rel", "stylesheet")
                    .attr("type", "text/css")
                    .attr("href", &format!("../{}", href)),
            )?;
            writer.write(XmlEvent::end_element())?;
        }

        writer.write(
            XmlEvent::start_element("meta")
                .attr("name", "viewport")
                .attr("content", &format!("width={}, height={}", width, height)),
        )?;
        writer.write(XmlEvent::end_element())?;

        writer.write(XmlEvent::end_element())?; // head

        {
            let mut event = XmlEvent::start_element("body");
            if cover {
                event = event.attr("epub:type", "cover");
            }
            writer.write(event)?;
        }

        writer.write(XmlEvent::start_element("div").attr("class", "main"))?;

        writer.write(
            XmlEvent::start_element("svg")
                .default_ns("http://www.w3.org/2000/svg")
                .ns("xlink", "http://www.w3.org/1999/xlink")
                .attr("version", "1.1")
                .attr("width", "100%")
                .attr("height", "100%")
                .attr("viewBox", &format!("0 0 {} {}", width, height)),
        )?;

        writer.write(
            XmlEvent::start_element("image")
                .attr("width", &width)
                .attr("height", &height)
                .attr("xlink:href", &format!("../{}", item.href)),
        )?;
        writer.write(XmlEvent::end_element())?;

        writer.write(XmlEvent::end_element())?; // svg
        writer.write(XmlEvent::end_element())?; // div
        writer.write(XmlEvent::end_element())?; // body
        writer.write(XmlEvent::end_element())?; // html

        Ok(file.into_temp_path())
    }
}

trait ToId {
    fn to_id(&self) -> String;
}

impl<T: AsRef<Path>> ToId for T {
    fn to_id(&self) -> String {
        let path = self.as_ref();

        let mut parts: Vec<&str> = path
            .parent()
            .map(|p| p.iter().map(|p| p.to_str().unwrap()).collect())
            .unwrap_or_default();

        if let Some(file_stem) = path.file_stem() {
            parts.push(file_stem.to_str().unwrap());
        }

        parts.join("-")
    }
}
