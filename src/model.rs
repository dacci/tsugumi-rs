use serde::de::{self, value::Error as ValueError};
use std::path::PathBuf;
use std::str::FromStr;

#[derive(Debug, Default, serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields, default)]
#[cfg_attr(test, derive(PartialEq))]
pub struct Book {
    pub metadata: Metadata,
    pub rendition: Rendition,
    pub chapter: Vec<Chapter>,
}

#[derive(Debug, Default, serde::Deserialize, serde::Serialize)]
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

#[derive(Debug, Default, serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields, default)]
#[cfg_attr(test, derive(PartialEq))]
pub struct Title {
    pub name: String,

    #[serde(rename = "type", with = "serde_enum")]
    pub title_type: TitleType,

    pub alternate_script: Option<String>,
    pub file_as: Option<String>,
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

#[derive(Debug, Default, serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields, default)]
#[cfg_attr(test, derive(PartialEq))]
pub struct Creator {
    pub name: String,
    pub role: Option<String>,
    pub alternate_script: Option<String>,
    pub file_as: Option<String>,
}

#[derive(Debug, serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
#[cfg_attr(test, derive(PartialEq))]
pub struct Collection {
    pub name: String,

    #[serde(rename = "type", with = "serde_enum")]
    pub collection_type: CollectionType,

    pub position: Option<u32>,
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

#[derive(Debug, Default, serde::Deserialize, serde::Serialize)]
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

#[derive(Debug, Default, serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields, default)]
#[cfg_attr(test, derive(PartialEq))]
pub struct Style {
    pub link: bool,
    pub href: String,
    pub src: String,
}

#[derive(Debug, Default, serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields, default)]
#[cfg_attr(test, derive(PartialEq))]
pub struct Chapter {
    pub title: Option<String>,
    pub page: Vec<Page>,
    pub cover: bool,
}

#[derive(Debug, Default, serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields, default)]
#[cfg_attr(test, derive(PartialEq))]
pub struct Page {
    pub src: PathBuf,
}

mod serde_enum {
    use serde::{de, ser};
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
