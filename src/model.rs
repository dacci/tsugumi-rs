use serde::de::{self, value::Error as ValueError};
use serde::ser::{self, SerializeMap};
use std::fmt;
use std::path::PathBuf;
use std::str::FromStr;

#[derive(Debug, Default)]
#[cfg_attr(test, derive(PartialEq))]
pub struct Book {
    pub metadata: Metadata,
    pub rendition: Rendition,
    pub chapter: Vec<Chapter>,
}

impl<'de> de::Deserialize<'de> for Book {
    fn deserialize<D: de::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        struct Visitor;

        impl<'de> de::Visitor<'de> for Visitor {
            type Value = Book;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("a map")
            }

            fn visit_map<A: de::MapAccess<'de>>(self, mut map: A) -> Result<Self::Value, A::Error> {
                enum Field {
                    Metadata,
                    Rendition,
                    Chapter,
                }

                impl<'de> de::Deserialize<'de> for Field {
                    fn deserialize<D: de::Deserializer<'de>>(
                        deserializer: D,
                    ) -> Result<Self, D::Error> {
                        struct Visitor;

                        impl<'de> de::Visitor<'de> for Visitor {
                            type Value = Field;

                            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                                formatter.write_str("an identifier")
                            }

                            fn visit_str<E: de::Error>(self, v: &str) -> Result<Self::Value, E> {
                                match v {
                                    "metadata" => Ok(Field::Metadata),
                                    "rendition" => Ok(Field::Rendition),
                                    "chapter" => Ok(Field::Chapter),
                                    field => Err(de::Error::unknown_field(
                                        field,
                                        &["metadata", "rendition", "chapter"],
                                    )),
                                }
                            }
                        }

                        deserializer.deserialize_identifier(Visitor)
                    }
                }

                let mut metadata = None;
                let mut rendition = None;
                let mut chapter = None;

                while let Some(field) = map.next_key()? {
                    match field {
                        Field::Metadata => {
                            if metadata.is_some() {
                                return Err(de::Error::duplicate_field("metadata"));
                            }
                            metadata = map.next_value().map(Some)?;
                        }
                        Field::Rendition => {
                            if rendition.is_some() {
                                return Err(de::Error::duplicate_field("rendition"));
                            }
                            rendition = map.next_value().map(Some)?;
                        }
                        Field::Chapter => {
                            if chapter.is_some() {
                                return Err(de::Error::duplicate_field("chapter"));
                            }
                            chapter = map
                                .next_value::<invariable::Deserialize<_>>()
                                .map(|d| d.unwrap())
                                .map(Some)?;
                        }
                    }
                }

                let metadata = metadata.ok_or_else(|| de::Error::missing_field("metadata"))?;
                let rendition = rendition.unwrap_or_default();
                let chapter = chapter.unwrap_or_default();

                Ok(Book {
                    metadata,
                    rendition,
                    chapter,
                })
            }
        }

        deserializer.deserialize_map(Visitor)
    }
}

impl ser::Serialize for Book {
    fn serialize<S: ser::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut map = serializer.serialize_map(None)?;

        map.serialize_entry("metadata", &self.metadata)?;
        map.serialize_entry("rendition", &self.rendition)?;

        if !self.chapter.is_empty() {
            map.serialize_entry("chapter", &invariable::wrap(&self.chapter))?;
        }

        map.end()
    }
}

#[derive(Debug, Default)]
#[cfg_attr(test, derive(PartialEq))]
pub struct Metadata {
    pub title: Vec<Title>,
    pub creator: Vec<Creator>,
    pub contributor: Vec<Creator>,
    pub collection: Vec<Collection>,
    pub language: String,
    pub identifier: String,
}

impl<'de> de::Deserialize<'de> for Metadata {
    fn deserialize<D: de::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        struct Visitor;

        impl<'de> de::Visitor<'de> for Visitor {
            type Value = Metadata;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("a map")
            }

            fn visit_map<A: de::MapAccess<'de>>(self, mut map: A) -> Result<Self::Value, A::Error> {
                enum Field {
                    Title,
                    Creator,
                    Contributor,
                    Collection,
                    Language,
                    Identifier,
                }

                impl<'de> de::Deserialize<'de> for Field {
                    fn deserialize<D: de::Deserializer<'de>>(
                        deserializer: D,
                    ) -> Result<Self, D::Error> {
                        struct Visitor;

                        impl<'de> de::Visitor<'de> for Visitor {
                            type Value = Field;

                            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                                formatter.write_str("an identifier")
                            }

                            fn visit_str<E: de::Error>(self, v: &str) -> Result<Self::Value, E> {
                                match v {
                                    "title" => Ok(Field::Title),
                                    "creator" => Ok(Field::Creator),
                                    "contributor" => Ok(Field::Contributor),
                                    "collection" => Ok(Field::Collection),
                                    "language" => Ok(Field::Language),
                                    "identifier" => Ok(Field::Identifier),
                                    field => Err(de::Error::unknown_field(
                                        field,
                                        &[
                                            "title",
                                            "creator",
                                            "contributor",
                                            "collection",
                                            "identifier",
                                        ],
                                    )),
                                }
                            }
                        }

                        deserializer.deserialize_identifier(Visitor)
                    }
                }

                let mut title = None;
                let mut creator = None;
                let mut contributor = None;
                let mut collection = None;
                let mut language = None;
                let mut identifier = None;

                while let Some(field) = map.next_key()? {
                    match field {
                        Field::Title => {
                            if title.is_some() {
                                return Err(de::Error::duplicate_field("title"));
                            }
                            title = map
                                .next_value::<invariable::Deserialize<_>>()
                                .map(|d| d.unwrap())
                                .map(Some)?;
                        }
                        Field::Creator => {
                            if creator.is_some() {
                                return Err(de::Error::duplicate_field("creator"));
                            }
                            creator = map
                                .next_value::<invariable::Deserialize<_>>()
                                .map(|d| d.unwrap())
                                .map(Some)?;
                        }
                        Field::Contributor => {
                            if contributor.is_some() {
                                return Err(de::Error::duplicate_field("contributor"));
                            }
                            contributor = map
                                .next_value::<invariable::Deserialize<_>>()
                                .map(|d| d.unwrap())
                                .map(Some)?;
                        }
                        Field::Collection => {
                            if collection.is_some() {
                                return Err(de::Error::duplicate_field("collection"));
                            }
                            collection = map
                                .next_value::<invariable::Deserialize<_>>()
                                .map(|d| d.unwrap())
                                .map(Some)?;
                        }
                        Field::Language => {
                            if language.is_some() {
                                return Err(de::Error::duplicate_field("language"));
                            }
                            language = map.next_value().map(Some)?;
                        }
                        Field::Identifier => {
                            if identifier.is_some() {
                                return Err(de::Error::duplicate_field("identifier"));
                            }
                            identifier = map.next_value().map(Some)?;
                        }
                    }
                }

                let title = title.unwrap_or_default();
                let creator = creator.unwrap_or_default();
                let contributor = contributor.unwrap_or_default();
                let collection = collection.unwrap_or_default();
                let language = language.ok_or_else(|| de::Error::missing_field("language"))?;
                let identifier =
                    identifier.ok_or_else(|| de::Error::missing_field("identifier"))?;

                Ok(Metadata {
                    title,
                    creator,
                    contributor,
                    collection,
                    language,
                    identifier,
                })
            }
        }

        deserializer.deserialize_map(Visitor)
    }
}

impl ser::Serialize for Metadata {
    fn serialize<S: ser::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut map = serializer.serialize_map(None)?;

        if !self.title.is_empty() {
            map.serialize_entry("title", &invariable::wrap(&self.title))?;
        }

        if !self.creator.is_empty() {
            map.serialize_entry("creator", &invariable::wrap(&self.creator))?;
        }

        if !self.contributor.is_empty() {
            map.serialize_entry("contributor", &invariable::wrap(&self.contributor))?;
        }

        if !self.collection.is_empty() {
            map.serialize_entry("collection", &invariable::wrap(&self.collection))?;
        }

        map.serialize_entry("language", &self.language)?;
        map.serialize_entry("identifier", &self.identifier)?;

        map.end()
    }
}

#[derive(Debug, Default, serde::Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields, default)]
#[cfg_attr(test, derive(PartialEq))]
pub struct Title {
    pub name: String,

    #[serde(rename = "type", with = "serde_enum")]
    pub title_type: TitleType,

    pub alternate_script: Option<String>,
    pub file_as: Option<String>,
}

impl ser::Serialize for Title {
    fn serialize<S: ser::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut map = serializer.serialize_map(None)?;

        map.serialize_entry("name", &self.name)?;

        if !self.title_type.is_default() {
            map.serialize_entry("type", &serde_enum::wrap(&self.title_type))?;
        }

        if let Some(alternate_script) = &self.alternate_script {
            map.serialize_entry("alternateScript", alternate_script)?;
        }

        if let Some(file_as) = &self.file_as {
            map.serialize_entry("fileAs", file_as)?;
        }

        map.end()
    }
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub enum TitleType {
    #[default]
    Main,
    Subtitle,
    Short,
    Collection,
    Edition,
    Expanded,
}

impl FromStr for TitleType {
    type Err = ValueError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "main" => Ok(Self::Main),
            "subtitle" => Ok(Self::Subtitle),
            "short" => Ok(Self::Short),
            "collection" => Ok(Self::Collection),
            "edition" => Ok(Self::Edition),
            "expanded" => Ok(Self::Expanded),
            variant => Err(de::Error::unknown_variant(
                variant,
                &[
                    "main",
                    "subtitle",
                    "short",
                    "collection",
                    "edition",
                    "expanded",
                ],
            )),
        }
    }
}

impl AsRef<str> for TitleType {
    fn as_ref(&self) -> &str {
        match self {
            Self::Main => "main",
            Self::Subtitle => "subtitle",
            Self::Short => "short",
            Self::Collection => "collection",
            Self::Edition => "edition",
            Self::Expanded => "expanded",
        }
    }
}

#[derive(Debug, Default, serde::Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields, default)]
#[cfg_attr(test, derive(PartialEq))]
pub struct Creator {
    pub name: String,
    pub role: Option<String>,
    pub alternate_script: Option<String>,
    pub file_as: Option<String>,
}

impl ser::Serialize for Creator {
    fn serialize<S: ser::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut map = serializer.serialize_map(None)?;

        map.serialize_entry("name", &self.name)?;

        if let Some(role) = &self.role {
            map.serialize_entry("role", role)?;
        }

        if let Some(alternate_script) = &self.alternate_script {
            map.serialize_entry("alternateScript", alternate_script)?;
        }

        if let Some(file_as) = &self.file_as {
            map.serialize_entry("fileAs", file_as)?;
        }

        map.end()
    }
}

#[derive(Debug, serde::Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
#[cfg_attr(test, derive(PartialEq))]
pub struct Collection {
    pub name: String,

    #[serde(rename = "type", with = "serde_enum")]
    pub collection_type: CollectionType,

    pub position: Option<u32>,
}

impl ser::Serialize for Collection {
    fn serialize<S: ser::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut map = serializer.serialize_map(None)?;

        map.serialize_entry("name", &self.name)?;
        map.serialize_entry("type", &serde_enum::wrap(&self.collection_type))?;

        if let Some(position) = &self.position {
            map.serialize_entry("position", position)?;
        }

        map.end()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CollectionType {
    Series,
    Set,
}

impl FromStr for CollectionType {
    type Err = ValueError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "series" => Ok(Self::Series),
            "set" => Ok(Self::Set),
            variant => Err(de::Error::unknown_variant(variant, &["series", "set"])),
        }
    }
}

impl AsRef<str> for CollectionType {
    fn as_ref(&self) -> &str {
        match self {
            Self::Series => "series",
            Self::Set => "set",
        }
    }
}

#[derive(Debug, Default)]
#[cfg_attr(test, derive(PartialEq))]
pub struct Rendition {
    pub direction: Direction,
    pub layout: Layout,
    pub orientation: Orientation,
    pub spread: Spread,
    pub style: Vec<Style>,
}

impl<'de> de::Deserialize<'de> for Rendition {
    fn deserialize<D: de::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        struct Visitor;

        impl<'de> de::Visitor<'de> for Visitor {
            type Value = Rendition;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("a map")
            }

            fn visit_map<A: de::MapAccess<'de>>(self, mut map: A) -> Result<Self::Value, A::Error> {
                enum Field {
                    Direction,
                    Layout,
                    Orientation,
                    Spread,
                    Style,
                }

                impl<'de> de::Deserialize<'de> for Field {
                    fn deserialize<D: de::Deserializer<'de>>(
                        deserializer: D,
                    ) -> Result<Self, D::Error> {
                        struct Visitor;

                        impl<'de> de::Visitor<'de> for Visitor {
                            type Value = Field;

                            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                                formatter.write_str("an identifier")
                            }

                            fn visit_str<E: de::Error>(self, v: &str) -> Result<Self::Value, E> {
                                match v {
                                    "direction" => Ok(Field::Direction),
                                    "layout" => Ok(Field::Layout),
                                    "orientation" => Ok(Field::Orientation),
                                    "spread" => Ok(Field::Spread),
                                    "style" => Ok(Field::Style),
                                    field => Err(de::Error::unknown_field(
                                        field,
                                        &["direction", "layout", "orientation", "spread", "style"],
                                    )),
                                }
                            }
                        }

                        deserializer.deserialize_identifier(Visitor)
                    }
                }

                let mut direction = None;
                let mut layout = None;
                let mut orientation = None;
                let mut spread = None;
                let mut style = None;

                while let Some(field) = map.next_key()? {
                    match field {
                        Field::Direction => {
                            if direction.is_some() {
                                return Err(de::Error::duplicate_field("direction"));
                            }
                            direction = map
                                .next_value::<serde_enum::Deserialize<_>>()
                                .map(|d| d.unwrap())
                                .map(Some)?;
                        }
                        Field::Layout => {
                            if layout.is_some() {
                                return Err(de::Error::duplicate_field("layout"));
                            }
                            layout = map
                                .next_value::<serde_enum::Deserialize<_>>()
                                .map(|d| d.unwrap())
                                .map(Some)?;
                        }
                        Field::Orientation => {
                            if orientation.is_some() {
                                return Err(de::Error::duplicate_field("orientation"));
                            }
                            orientation = map
                                .next_value::<serde_enum::Deserialize<_>>()
                                .map(|d| d.unwrap())
                                .map(Some)?;
                        }
                        Field::Spread => {
                            if spread.is_some() {
                                return Err(de::Error::duplicate_field("spread"));
                            }
                            spread = map
                                .next_value::<serde_enum::Deserialize<_>>()
                                .map(|d| d.unwrap())
                                .map(Some)?;
                        }
                        Field::Style => {
                            if style.is_some() {
                                return Err(de::Error::duplicate_field("style"));
                            }
                            style = map
                                .next_value::<invariable::Deserialize<_>>()
                                .map(|d| d.unwrap())
                                .map(Some)?;
                        }
                    }
                }

                let direction = direction.unwrap_or_default();
                let layout = layout.unwrap_or_default();
                let orientation = orientation.unwrap_or_default();
                let spread = spread.unwrap_or_default();
                let style = style.unwrap_or_default();

                Ok(Rendition {
                    direction,
                    layout,
                    orientation,
                    spread,
                    style,
                })
            }
        }

        deserializer.deserialize_map(Visitor)
    }
}

impl ser::Serialize for Rendition {
    fn serialize<S: ser::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut map = serializer.serialize_map(None)?;

        if !self.direction.is_default() {
            map.serialize_entry("direction", &serde_enum::wrap(&self.direction))?;
        }

        if !self.layout.is_default() {
            map.serialize_entry("layout", &serde_enum::wrap(&self.layout))?;
        }

        if !self.orientation.is_default() {
            map.serialize_entry("orientation", &serde_enum::wrap(&self.orientation))?;
        }

        if !self.spread.is_default() {
            map.serialize_entry("spread", &serde_enum::wrap(&self.spread))?;
        }

        if !self.style.is_empty() {
            map.serialize_entry("style", &invariable::wrap(&self.style))?;
        }

        map.end()
    }
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub enum Direction {
    #[default]
    RightToLeft,
    LeftToRight,
}

impl FromStr for Direction {
    type Err = ValueError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "rtl" => Ok(Self::RightToLeft),
            "ltr" => Ok(Self::LeftToRight),
            variant => Err(de::Error::unknown_variant(variant, &["rtl", "ltr"])),
        }
    }
}

impl AsRef<str> for Direction {
    fn as_ref(&self) -> &str {
        match self {
            Self::RightToLeft => "rtl",
            Self::LeftToRight => "ltr",
        }
    }
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub enum Layout {
    Reflowable,
    #[default]
    PrePaginated,
}

impl FromStr for Layout {
    type Err = ValueError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "reflowable" => Ok(Self::Reflowable),
            "pre-paginated" => Ok(Self::PrePaginated),
            variant => Err(de::Error::unknown_variant(
                variant,
                &["reflowable", "pre-paginated"],
            )),
        }
    }
}

impl AsRef<str> for Layout {
    fn as_ref(&self) -> &str {
        match self {
            Self::Reflowable => "reflowable",
            Self::PrePaginated => "pre-paginated",
        }
    }
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub enum Orientation {
    Landscape,
    Portrait,
    #[default]
    Auto,
}

impl FromStr for Orientation {
    type Err = ValueError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "landscape" => Ok(Self::Landscape),
            "portrait" => Ok(Self::Portrait),
            "auto" => Ok(Self::Auto),
            variant => Err(de::Error::unknown_variant(
                variant,
                &["landscape", "portrait", "auto"],
            )),
        }
    }
}

impl AsRef<str> for Orientation {
    fn as_ref(&self) -> &str {
        match self {
            Self::Landscape => "landscape",
            Self::Portrait => "portrait",
            Self::Auto => "auto",
        }
    }
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub enum Spread {
    None,
    Landscape,
    Both,
    #[default]
    Auto,
}

impl FromStr for Spread {
    type Err = ValueError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "none" => Ok(Self::None),
            "landscape" => Ok(Self::Landscape),
            "both" => Ok(Self::Both),
            "auto" => Ok(Self::Auto),
            variant => Err(de::Error::unknown_variant(
                variant,
                &["none", "landscape", "both", "auto"],
            )),
        }
    }
}

impl AsRef<str> for Spread {
    fn as_ref(&self) -> &str {
        match self {
            Self::None => "none",
            Self::Landscape => "landscape",
            Self::Both => "both",
            Self::Auto => "auto",
        }
    }
}

#[derive(Debug, Default, serde::Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields, default)]
#[cfg_attr(test, derive(PartialEq))]
pub struct Style {
    pub link: bool,
    pub href: String,
    pub src: String,
}

impl ser::Serialize for Style {
    fn serialize<S: ser::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut map = serializer.serialize_map(None)?;

        if self.link {
            map.serialize_entry("link", &self.link)?;
        }

        map.serialize_entry("href", &self.href)?;
        map.serialize_entry("src", &self.src)?;

        map.end()
    }
}

#[derive(Debug, Default)]
#[cfg_attr(test, derive(PartialEq))]
pub struct Chapter {
    pub title: Option<String>,
    pub page: Vec<Page>,
    pub cover: bool,
}

impl<'de> de::Deserialize<'de> for Chapter {
    fn deserialize<D: de::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        struct Visitor;

        impl<'de> de::Visitor<'de> for Visitor {
            type Value = Chapter;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("a map")
            }

            fn visit_map<A: de::MapAccess<'de>>(self, mut map: A) -> Result<Self::Value, A::Error> {
                enum Field {
                    Title,
                    Page,
                    Cover,
                }

                impl<'de> de::Deserialize<'de> for Field {
                    fn deserialize<D: de::Deserializer<'de>>(
                        deserializer: D,
                    ) -> Result<Self, D::Error> {
                        struct Visitor;

                        impl<'de> de::Visitor<'de> for Visitor {
                            type Value = Field;

                            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                                formatter.write_str("an identifier")
                            }

                            fn visit_str<E: de::Error>(self, v: &str) -> Result<Self::Value, E> {
                                match v {
                                    "title" => Ok(Field::Title),
                                    "page" => Ok(Field::Page),
                                    "cover" => Ok(Field::Cover),
                                    field => Err(de::Error::unknown_field(
                                        field,
                                        &["title", "page", "cover"],
                                    )),
                                }
                            }
                        }

                        deserializer.deserialize_identifier(Visitor)
                    }
                }

                let mut title = None;
                let mut page = None;
                let mut cover = None;

                while let Some(field) = map.next_key()? {
                    match field {
                        Field::Title => {
                            if title.is_some() {
                                return Err(de::Error::duplicate_field("title"));
                            }
                            title = map.next_value().map(Some)?;
                        }
                        Field::Page => {
                            if page.is_some() {
                                return Err(de::Error::duplicate_field("page"));
                            }
                            page = map
                                .next_value::<invariable::Deserialize<_>>()
                                .map(|d| d.unwrap())
                                .map(Some)?;
                        }
                        Field::Cover => {
                            if cover.is_some() {
                                return Err(de::Error::duplicate_field("cover"));
                            }
                            cover = map.next_value().map(Some)?;
                        }
                    }
                }

                let title = title.unwrap_or_default();
                let page = page.unwrap_or_default();
                let cover = cover.unwrap_or_default();

                Ok(Chapter { title, page, cover })
            }
        }

        deserializer.deserialize_map(Visitor)
    }
}

impl ser::Serialize for Chapter {
    fn serialize<S: ser::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut map = serializer.serialize_map(None)?;

        if let Some(title) = &self.title {
            map.serialize_entry("title", title)?;
        }

        if !self.page.is_empty() {
            map.serialize_entry("page", &invariable::wrap(&self.page))?;
        }

        if self.cover {
            map.serialize_entry("cover", &self.cover)?;
        }

        map.end()
    }
}

#[derive(Debug, Default)]
#[cfg_attr(test, derive(PartialEq))]
pub struct Page {
    pub src: PathBuf,
}

impl<'de> de::Deserialize<'de> for Page {
    fn deserialize<D: de::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        <PathBuf as de::Deserialize>::deserialize(deserializer).map(|src| Page { src })
    }
}

impl ser::Serialize for Page {
    fn serialize<S: ser::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        ser::Serialize::serialize(&self.src, serializer)
    }
}

trait IsDefault {
    fn is_default(&self) -> bool;
}

impl<T: PartialEq + Default> IsDefault for T {
    fn is_default(&self) -> bool {
        T::default().eq(self)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_test::*;

    #[test]
    fn test_serde_book() {
        assert_tokens(
            &Book::default(),
            &[
                Token::Map { len: None },
                Token::Str("metadata"),
                Token::Map { len: None },
                Token::Str("language"),
                Token::Str(""),
                Token::Str("identifier"),
                Token::Str(""),
                Token::MapEnd,
                Token::Str("rendition"),
                Token::Map { len: None },
                Token::MapEnd,
                Token::MapEnd,
            ],
        );

        assert_tokens(
            &Book {
                chapter: vec![Chapter::default()],
                ..Book::default()
            },
            &[
                Token::Map { len: None },
                Token::Str("metadata"),
                Token::Map { len: None },
                Token::Str("language"),
                Token::Str(""),
                Token::Str("identifier"),
                Token::Str(""),
                Token::MapEnd,
                Token::Str("rendition"),
                Token::Map { len: None },
                Token::MapEnd,
                Token::Str("chapter"),
                Token::Map { len: None },
                Token::MapEnd,
                Token::MapEnd,
            ],
        );
    }

    #[test]
    fn test_serde_metadata() {
        assert_tokens(
            &Metadata::default(),
            &[
                Token::Map { len: None },
                Token::Str("language"),
                Token::Str(""),
                Token::Str("identifier"),
                Token::Str(""),
                Token::MapEnd,
            ],
        );
        assert_tokens(
            &Metadata {
                title: vec![Title::default()],
                ..Metadata::default()
            },
            &[
                Token::Map { len: None },
                Token::Str("title"),
                Token::Map { len: None },
                Token::Str("name"),
                Token::Str(""),
                Token::MapEnd,
                Token::Str("language"),
                Token::Str(""),
                Token::Str("identifier"),
                Token::Str(""),
                Token::MapEnd,
            ],
        );

        assert_de_tokens_error::<Metadata>(
            &[Token::Map { len: Some(0) }, Token::MapEnd],
            "missing field `language`",
        );
    }

    #[test]
    fn test_serde_title() {
        assert_tokens(
            &Title::default(),
            &[
                Token::Map { len: None },
                Token::Str("name"),
                Token::Str(""),
                Token::MapEnd,
            ],
        );
    }

    #[test]
    fn test_serde_creator() {
        assert_tokens(
            &Creator::default(),
            &[
                Token::Map { len: None },
                Token::Str("name"),
                Token::Str(""),
                Token::MapEnd,
            ],
        );
    }

    #[test]
    fn test_serde_collection() {
        assert_tokens(
            &Collection {
                name: Default::default(),
                collection_type: CollectionType::Series,
                position: Default::default(),
            },
            &[
                Token::Map { len: None },
                Token::Str("name"),
                Token::Str(""),
                Token::Str("type"),
                Token::Str("series"),
                Token::MapEnd,
            ],
        );
    }

    #[test]
    fn test_serde_rendition() {
        assert_tokens(
            &Rendition::default(),
            &[Token::Map { len: None }, Token::MapEnd],
        );
        assert_tokens(
            &Rendition {
                style: vec![Style::default()],
                ..Rendition::default()
            },
            &[
                Token::Map { len: None },
                Token::Str("style"),
                Token::Map { len: None },
                Token::Str("href"),
                Token::Str(""),
                Token::Str("src"),
                Token::Str(""),
                Token::MapEnd,
                Token::MapEnd,
            ],
        );
    }

    #[test]
    fn test_serde_style() {
        assert_tokens(
            &Style::default(),
            &[
                Token::Map { len: None },
                Token::Str("href"),
                Token::Str(""),
                Token::Str("src"),
                Token::Str(""),
                Token::MapEnd,
            ],
        );
    }

    #[test]
    fn test_serde_chapter() {
        assert_tokens(
            &Chapter::default(),
            &[Token::Map { len: None }, Token::MapEnd],
        );
        assert_tokens(
            &Chapter {
                page: vec![Page::default()],
                ..Chapter::default()
            },
            &[
                Token::Map { len: None },
                Token::Str("page"),
                Token::Str(""),
                Token::MapEnd,
            ],
        );
    }

    #[test]
    fn test_serde_page() {
        assert_tokens(&Page { src: "path".into() }, &[Token::Str("path")]);
    }
}

mod serde_enum {
    use super::*;
    use std::error::Error;
    use std::fmt;
    use std::marker::PhantomData;
    use std::str::FromStr;

    struct Visitor<T>(PhantomData<T>);

    impl<'de, T> de::Visitor<'de> for Visitor<T>
    where
        T: FromStr,
        T::Err: Error,
    {
        type Value = T;

        fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
            formatter.write_str("a string")
        }

        fn visit_str<E: de::Error>(self, v: &str) -> Result<Self::Value, E> {
            v.parse().map_err(de::Error::custom)
        }
    }

    pub fn deserialize<'de, D, T>(deserializer: D) -> Result<T, D::Error>
    where
        D: de::Deserializer<'de>,
        T: FromStr,
        T::Err: Error,
    {
        deserializer.deserialize_str(Visitor(PhantomData))
    }

    pub struct Deserialize<T>(T);

    impl<T> Deserialize<T> {
        pub fn unwrap(self) -> T {
            self.0
        }
    }

    impl<'de, T> de::Deserialize<'de> for Deserialize<T>
    where
        T: FromStr,
        T::Err: Error,
    {
        fn deserialize<D: de::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
            deserialize(deserializer).map(Self)
        }
    }

    pub fn serialize<T, S>(v: &T, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: ser::Serializer,
        T: AsRef<str>,
    {
        serializer.serialize_str(v.as_ref())
    }

    pub struct Serialize<'a, T>(&'a T);

    impl<T: AsRef<str>> ser::Serialize for Serialize<'_, T> {
        fn serialize<S: ser::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
            serialize(&self.0, serializer)
        }
    }

    pub fn wrap<T>(inner: &T) -> Serialize<T> {
        Serialize(inner)
    }

    #[cfg(test)]
    mod tests {
        use super::*;
        use serde_test::{assert_tokens, Token};

        #[derive(Debug, PartialEq, serde::Deserialize, serde::Serialize)]
        struct Test {
            #[serde(with = "super")]
            foo_bar: FooBar,
        }

        #[derive(Debug, PartialEq)]
        enum FooBar {
            Foo,
            Bar,
        }

        impl FromStr for FooBar {
            type Err = de::value::Error;

            fn from_str(s: &str) -> Result<Self, Self::Err> {
                match s {
                    "foo" => Ok(Self::Foo),
                    "bar" => Ok(Self::Bar),
                    variant => Err(de::Error::unknown_variant(variant, &["foo", "bar"])),
                }
            }
        }

        impl AsRef<str> for FooBar {
            fn as_ref(&self) -> &str {
                match self {
                    FooBar::Foo => "foo",
                    FooBar::Bar => "bar",
                }
            }
        }

        #[test]
        fn test_serde() {
            assert_tokens(
                &Test {
                    foo_bar: FooBar::Foo,
                },
                &[
                    Token::Struct {
                        name: "Test",
                        len: 1,
                    },
                    Token::Str("foo_bar"),
                    Token::Str("foo"),
                    Token::StructEnd,
                ],
            );
        }
    }
}

mod invariable {
    use serde::de::{self, value};
    use serde::ser;
    use std::fmt;
    use std::marker::PhantomData;

    struct Visitor<T>(PhantomData<T>);

    impl<'de, T: de::Deserialize<'de>> de::Visitor<'de> for Visitor<T> {
        type Value = Vec<T>;

        fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
            formatter.write_str("supported data types")
        }

        fn visit_bool<E: de::Error>(self, v: bool) -> Result<Self::Value, E> {
            de::Deserialize::deserialize(value::BoolDeserializer::new(v)).map(|e| vec![e])
        }

        fn visit_i8<E: de::Error>(self, v: i8) -> Result<Self::Value, E> {
            de::Deserialize::deserialize(value::I8Deserializer::new(v)).map(|e| vec![e])
        }

        fn visit_i16<E: de::Error>(self, v: i16) -> Result<Self::Value, E> {
            de::Deserialize::deserialize(value::I16Deserializer::new(v)).map(|e| vec![e])
        }

        fn visit_i32<E: de::Error>(self, v: i32) -> Result<Self::Value, E> {
            de::Deserialize::deserialize(value::I32Deserializer::new(v)).map(|e| vec![e])
        }

        fn visit_i64<E: de::Error>(self, v: i64) -> Result<Self::Value, E> {
            de::Deserialize::deserialize(value::I64Deserializer::new(v)).map(|e| vec![e])
        }

        fn visit_i128<E: de::Error>(self, v: i128) -> Result<Self::Value, E> {
            de::Deserialize::deserialize(value::I128Deserializer::new(v)).map(|e| vec![e])
        }

        fn visit_u8<E: de::Error>(self, v: u8) -> Result<Self::Value, E> {
            de::Deserialize::deserialize(value::U8Deserializer::new(v)).map(|e| vec![e])
        }

        fn visit_u16<E: de::Error>(self, v: u16) -> Result<Self::Value, E> {
            de::Deserialize::deserialize(value::U16Deserializer::new(v)).map(|e| vec![e])
        }

        fn visit_u32<E: de::Error>(self, v: u32) -> Result<Self::Value, E> {
            de::Deserialize::deserialize(value::U32Deserializer::new(v)).map(|e| vec![e])
        }

        fn visit_u64<E: de::Error>(self, v: u64) -> Result<Self::Value, E> {
            de::Deserialize::deserialize(value::U64Deserializer::new(v)).map(|e| vec![e])
        }

        fn visit_u128<E: de::Error>(self, v: u128) -> Result<Self::Value, E> {
            de::Deserialize::deserialize(value::U128Deserializer::new(v)).map(|e| vec![e])
        }

        fn visit_f32<E: de::Error>(self, v: f32) -> Result<Self::Value, E> {
            de::Deserialize::deserialize(value::F32Deserializer::new(v)).map(|e| vec![e])
        }

        fn visit_f64<E: de::Error>(self, v: f64) -> Result<Self::Value, E> {
            de::Deserialize::deserialize(value::F64Deserializer::new(v)).map(|e| vec![e])
        }

        fn visit_char<E: de::Error>(self, v: char) -> Result<Self::Value, E> {
            de::Deserialize::deserialize(value::CharDeserializer::new(v)).map(|e| vec![e])
        }

        fn visit_str<E: de::Error>(self, v: &str) -> Result<Self::Value, E> {
            de::Deserialize::deserialize(value::StrDeserializer::new(v)).map(|e| vec![e])
        }

        fn visit_borrowed_str<E: de::Error>(self, v: &'de str) -> Result<Self::Value, E> {
            de::Deserialize::deserialize(value::BorrowedStrDeserializer::new(v)).map(|e| vec![e])
        }

        fn visit_string<E: de::Error>(self, v: String) -> Result<Self::Value, E> {
            de::Deserialize::deserialize(value::StringDeserializer::new(v)).map(|e| vec![e])
        }

        fn visit_bytes<E: de::Error>(self, v: &[u8]) -> Result<Self::Value, E> {
            de::Deserialize::deserialize(value::BytesDeserializer::new(v)).map(|e| vec![e])
        }

        fn visit_borrowed_bytes<E: de::Error>(self, v: &'de [u8]) -> Result<Self::Value, E> {
            de::Deserialize::deserialize(value::BorrowedBytesDeserializer::new(v)).map(|e| vec![e])
        }

        fn visit_seq<A: de::SeqAccess<'de>>(self, seq: A) -> Result<Self::Value, A::Error> {
            de::Deserialize::deserialize(value::SeqAccessDeserializer::new(seq))
        }

        fn visit_map<A: de::MapAccess<'de>>(self, map: A) -> Result<Self::Value, A::Error> {
            de::Deserialize::deserialize(value::MapAccessDeserializer::new(map)).map(|e| vec![e])
        }
    }

    #[cfg_attr(test, derive(Debug, PartialEq))]
    pub struct Deserialize<T>(Vec<T>);

    impl<T> Deserialize<T> {
        pub fn unwrap(self) -> Vec<T> {
            self.0
        }
    }

    impl<'de, T: de::Deserialize<'de>> de::Deserialize<'de> for Deserialize<T> {
        fn deserialize<D: de::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
            deserializer.deserialize_any(Visitor(PhantomData)).map(Self)
        }
    }

    pub struct Serialize<'a, T>(&'a [T]);

    impl<T: ser::Serialize> ser::Serialize for Serialize<'_, T> {
        fn serialize<S: ser::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
            if self.0.len() == 1 {
                ser::Serialize::serialize(&self.0[0], serializer)
            } else {
                serializer.collect_seq(self.0)
            }
        }
    }

    pub fn wrap<T>(inner: &[T]) -> Serialize<T> {
        Serialize(inner)
    }

    #[cfg(test)]
    mod tests {
        use super::*;
        use serde_test::*;

        #[test]
        fn test_de() {
            assert_de_tokens(
                &Deserialize::<i32>(vec![]),
                &[Token::Seq { len: Some(0) }, Token::SeqEnd],
            );
            assert_de_tokens(&Deserialize::<i32>(vec![1]), &[Token::I32(1)]);
            assert_de_tokens(
                &Deserialize::<i32>(vec![1, 2]),
                &[
                    Token::Seq { len: Some(2) },
                    Token::I32(1),
                    Token::I32(2),
                    Token::SeqEnd,
                ],
            );

            assert_de_tokens_error::<Deserialize<i32>>(
                &[Token::Unit],
                "invalid type: unit value, expected supported data types",
            );
        }

        #[test]
        fn test_ser() {
            assert_ser_tokens(
                &Serialize::<i32>(&vec![]),
                &[Token::Seq { len: Some(0) }, Token::SeqEnd],
            );
            assert_ser_tokens(&Serialize(&vec![1]), &[Token::I32(1)]);
            assert_ser_tokens(
                &Serialize(&vec![1, 2]),
                &[
                    Token::Seq { len: Some(2) },
                    Token::I32(1),
                    Token::I32(2),
                    Token::SeqEnd,
                ],
            );
        }
    }
}
