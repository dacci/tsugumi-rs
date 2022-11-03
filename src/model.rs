use serde::de::{self, value::Error as ValueError};
use serde::ser::{self, SerializeMap};
use std::path::PathBuf;
use std::str::FromStr;

#[derive(Debug, Default, serde::Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields, default)]
#[cfg_attr(test, derive(PartialEq))]
pub struct Book {
    pub metadata: Metadata,
    pub rendition: Rendition,
    pub chapter: Vec<Chapter>,
}

impl ser::Serialize for Book {
    fn serialize<S: ser::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut map = serializer.serialize_map(None)?;

        map.serialize_entry("metadata", &self.metadata)?;
        map.serialize_entry("rendition", &self.rendition)?;

        if !self.chapter.is_empty() {
            map.serialize_entry("chapter", &self.chapter)?;
        }

        map.end()
    }
}

#[derive(Debug, Default, serde::Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields, default)]
#[cfg_attr(test, derive(PartialEq))]
pub struct Metadata {
    pub title: Vec<Title>,
    pub creator: Vec<Creator>,
    pub contributor: Vec<Creator>,
    pub collection: Vec<Collection>,
    pub language: String,
    pub identifier: String,
}

impl ser::Serialize for Metadata {
    fn serialize<S: ser::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut map = serializer.serialize_map(None)?;

        if !self.title.is_empty() {
            map.serialize_entry("title", &self.title)?;
        }

        if !self.creator.is_empty() {
            map.serialize_entry("creator", &self.creator)?;
        }

        if !self.contributor.is_empty() {
            map.serialize_entry("contributor", &self.contributor)?;
        }

        if !self.collection.is_empty() {
            map.serialize_entry("collection", &self.collection)?;
        }

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
            map.serialize_entry("type", &serde_enum::Serialize(&self.title_type))?;
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
        map.serialize_entry("type", &serde_enum::Serialize(&self.collection_type))?;

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

#[derive(Debug, Default, serde::Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields, default)]
#[cfg_attr(test, derive(PartialEq))]
pub struct Rendition {
    #[serde(with = "serde_enum")]
    pub direction: Direction,

    #[serde(with = "serde_enum")]
    pub layout: Layout,

    #[serde(with = "serde_enum")]
    pub orientation: Orientation,

    #[serde(with = "serde_enum")]
    pub spread: Spread,

    pub style: Vec<Style>,
}

impl ser::Serialize for Rendition {
    fn serialize<S: ser::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut map = serializer.serialize_map(None)?;

        if !self.direction.is_default() {
            map.serialize_entry("direction", &serde_enum::Serialize(&self.direction))?;
        }

        if !self.layout.is_default() {
            map.serialize_entry("layout", &serde_enum::Serialize(&self.layout))?;
        }

        if !self.orientation.is_default() {
            map.serialize_entry("orientation", &serde_enum::Serialize(&self.orientation))?;
        }

        if !self.spread.is_default() {
            map.serialize_entry("spread", &serde_enum::Serialize(&self.spread))?;
        }

        if !self.style.is_empty() {
            map.serialize_entry("style", &self.style)?;
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

#[derive(Debug, Default, serde::Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields, default)]
#[cfg_attr(test, derive(PartialEq))]
pub struct Chapter {
    pub title: Option<String>,
    pub page: Vec<Page>,
    pub cover: bool,
}

impl ser::Serialize for Chapter {
    fn serialize<S: ser::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut map = serializer.serialize_map(None)?;

        if let Some(title) = &self.title {
            map.serialize_entry("title", title)?;
        }

        if !self.page.is_empty() {
            map.serialize_entry("page", &self.page)?;
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
                Token::Str("identifier"),
                Token::Str(""),
                Token::MapEnd,
                Token::Str("rendition"),
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
                Token::Str("identifier"),
                Token::Str(""),
                Token::MapEnd,
            ],
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

    pub fn deserialize<'de, D, T>(deserializer: D) -> Result<T, D::Error>
    where
        D: de::Deserializer<'de>,
        T: FromStr,
        T::Err: Error,
    {
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

        deserializer.deserialize_str(Visitor(PhantomData))
    }

    pub fn serialize<T, S>(v: &T, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: ser::Serializer,
        T: AsRef<str>,
    {
        serializer.serialize_str(v.as_ref())
    }

    pub struct Serialize<'a, T>(pub &'a T);

    impl<T: AsRef<str>> ser::Serialize for Serialize<'_, T> {
        fn serialize<S: ser::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
            serialize(&self.0, serializer)
        }
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
