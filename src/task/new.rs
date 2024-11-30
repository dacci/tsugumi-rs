use crate::model::{
    Book, Chapter, Creator, Metadata, Orientation, Page, Rendition, Title, TitleType,
};
use anyhow::Result;
use std::fs::File;
use std::path::PathBuf;

#[derive(clap::Args)]
pub(super) struct Args {
    /// Set the main title of the book.
    #[arg(short, long)]
    title: Option<String>,

    /// Set the author of the book.
    #[arg(short, long)]
    author: Option<String>,

    /// Set the identifier of the book.
    #[arg(short, long, value_name = "URN")]
    identifier: Option<String>,

    /// Create pages from files and set the first page as the cover page.
    files: Vec<PathBuf>,
}

pub(super) fn main(args: Args) -> Result<()> {
    let metadata = Metadata {
        title: vec![Title {
            name: args.title.as_ref().cloned().unwrap_or_else(|| {
                std::env::current_dir()
                    .unwrap_or_default()
                    .file_name()
                    .unwrap_or_default()
                    .to_string_lossy()
                    .to_string()
            }),
            title_type: TitleType::Main,
            ..Default::default()
        }],
        creator: args
            .author
            .map(|name| Creator {
                name,
                role: Some("aut".to_string()),
                ..Default::default()
            })
            .map(|c| vec![c])
            .unwrap_or_default(),
        language: std::env::var("LANG")
            .ok()
            .as_deref()
            .and_then(|l| l.split('_').next())
            .unwrap_or("ja")
            .to_string(),
        identifier: args
            .identifier
            .unwrap_or_else(|| format!("urn:uuid:{}", uuid::Uuid::new_v4())),
        ..Default::default()
    };

    let rendition = Rendition {
        orientation: Orientation::Portrait,
        ..Default::default()
    };

    let book = Book {
        metadata,
        rendition,
        chapter: create_chapter(args.title.as_deref(), &args.files),
    };

    let file = File::create("tsugumi.yaml")?;
    serde_yaml::to_writer(file, &book)?;

    Ok(())
}

fn create_chapter(title: Option<&str>, files: &[PathBuf]) -> Vec<Chapter> {
    let mut iter = files.iter().map(|src| Page { src: src.clone() });
    let cover = iter.next().map(|page| Chapter {
        name: Some("表紙".to_string()),
        page: vec![page],
        cover: true,
    });
    let pages = Chapter {
        name: title.map(|s| s.to_string()),
        page: iter.collect::<Vec<_>>(),
        ..Default::default()
    };

    cover.into_iter().chain(Some(pages)).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_into_chapter() {
        let mut iter = create_chapter(
            Some("title"),
            &["cover".into(), "page1".into(), "page2".into()],
        )
        .into_iter();
        assert_eq!(
            iter.next(),
            Some(Chapter {
                name: Some("表紙".to_string()),
                page: vec![Page {
                    src: "cover".into()
                }],
                cover: true,
            })
        );
        assert_eq!(
            iter.next(),
            Some(Chapter {
                name: Some("title".to_string()),
                page: vec![
                    Page {
                        src: "page1".into()
                    },
                    Page {
                        src: "page2".into()
                    }
                ],
                ..Default::default()
            })
        );
        assert_eq!(iter.next(), None);
    }

    #[test]
    fn test_into_chapter_cover_only() {
        let mut iter = create_chapter(None, &["cover".into()]).into_iter();
        assert_eq!(
            iter.next(),
            Some(Chapter {
                name: Some("表紙".to_string()),
                page: vec![Page {
                    src: "cover".into()
                }],
                cover: true,
            })
        );
        assert_eq!(iter.next(), Some(Default::default()));
        assert_eq!(iter.next(), None);
    }

    #[test]
    fn test_into_chapter_empty() {
        let mut iter = create_chapter(None, &[]).into_iter();
        assert_eq!(iter.next(), Some(Default::default()));
        assert_eq!(iter.next(), None);
    }
}
