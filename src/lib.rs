pub mod ebpaj;

use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Deserialize, Serialize)]
pub struct Metadata {
    pub title: String,
    pub author: String,
}

#[derive(Deserialize, Serialize)]
pub struct Chapter {
    pub name: Option<String>,
    #[serde(default)]
    pub pages: Vec<PathBuf>,
}

#[derive(Deserialize, Serialize)]
pub struct Book {
    pub metadata: Metadata,
    pub cover: PathBuf,
    #[serde(default)]
    pub chapters: Vec<Chapter>,
}

#[derive(Deserialize, Serialize)]
pub struct Style {
    #[serde(default)]
    pub links: Vec<PathBuf>,
    #[serde(default)]
    pub includes: Vec<PathBuf>,
}
