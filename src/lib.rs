pub mod ebpaj;

use serde::de;
use serde::ser;
use serde::{Deserialize, Serialize};
use std::fmt;
use std::path::PathBuf;

#[derive(Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Book {
    pub metadata: Metadata,
    pub cover: PathBuf,
    #[serde(default)]
    pub chapters: Vec<Chapter>,
}

#[derive(Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Metadata {
    pub title: String,
    pub author: String,
}

#[derive(Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Chapter {
    pub name: Option<String>,
    #[serde(default)]
    pub pages: Vec<Page>,
}

#[derive(Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Page {
    pub path: PathBuf,
    pub spread: Option<Spread>,
}

impl<'de> de::Deserialize<'de> for Page {
    fn deserialize<D: de::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        enum Field {
            Path,
            Spread,
        }

        const FIELDS: &[&str] = &["path", "spread"];

        struct FieldVisitor;

        impl<'de> de::Visitor<'de> for FieldVisitor {
            type Value = Field;

            fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
                f.write_str("an identifier")
            }

            fn visit_str<E: de::Error>(self, v: &str) -> Result<Self::Value, E> {
                match v {
                    "path" => Ok(Field::Path),
                    "spread" => Ok(Field::Spread),
                    field => Err(de::Error::unknown_field(field, FIELDS)),
                }
            }
        }

        impl<'de> de::Deserialize<'de> for Field {
            fn deserialize<D: de::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
                deserializer.deserialize_identifier(FieldVisitor)
            }
        }

        struct Visitor;

        impl<'de> de::Visitor<'de> for Visitor {
            type Value = Page;

            fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
                f.write_str("a map or a string")
            }

            fn visit_str<E: de::Error>(self, v: &str) -> Result<Self::Value, E> {
                Ok(Page {
                    path: PathBuf::from(v),
                    spread: Default::default(),
                })
            }

            fn visit_string<E: de::Error>(self, v: String) -> Result<Self::Value, E> {
                Ok(Page {
                    path: PathBuf::from(v),
                    spread: Default::default(),
                })
            }

            fn visit_map<A: de::MapAccess<'de>>(self, mut map: A) -> Result<Self::Value, A::Error> {
                let mut path = None;
                let mut spread = None;

                while let Some(field) = map.next_key()? {
                    match field {
                        Field::Path => {
                            if path.is_some() {
                                return Err(de::Error::duplicate_field("path"));
                            }
                            path = Some(map.next_value()?);
                        }
                        Field::Spread => {
                            if spread.is_some() {
                                return Err(de::Error::duplicate_field("spread"));
                            }
                            spread = Some(map.next_value()?);
                        }
                    }
                }

                let path = path.ok_or_else(|| de::Error::missing_field("path"))?;

                Ok(Page { path, spread })
            }
        }

        deserializer.deserialize_any(Visitor)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Spread {
    Left,
    Right,
    Center,
}

impl<'de> de::Deserialize<'de> for Spread {
    fn deserialize<D: de::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        struct Visitor;

        impl<'de> de::Visitor<'de> for Visitor {
            type Value = Spread;

            fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
                f.write_str("a string")
            }

            fn visit_str<E: de::Error>(self, v: &str) -> Result<Self::Value, E> {
                let value = match v {
                    "left" => Spread::Left,
                    "right" => Spread::Right,
                    "center" => Spread::Center,
                    variant => {
                        return Err(de::Error::unknown_variant(
                            variant,
                            &["left", "right", "center"],
                        ))
                    }
                };
                Ok(value)
            }
        }

        deserializer.deserialize_identifier(Visitor)
    }
}

impl ser::Serialize for Spread {
    fn serialize<S: ser::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let s = match self {
            Spread::Left => "left",
            Spread::Right => "right",
            Spread::Center => "center",
        };
        serializer.serialize_str(s)
    }
}

impl Iterator for Spread {
    type Item = Spread;

    fn next(&mut self) -> Option<Self::Item> {
        match self {
            Spread::Left | Spread::Center => Some(Self::Right),
            Spread::Right => Some(Self::Left),
        }
    }
}

#[derive(Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Style {
    pub links: Vec<PathBuf>,
    #[serde(default)]
    pub includes: Vec<PathBuf>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_test::{assert_de_tokens, assert_de_tokens_error, assert_tokens, Token};

    #[test]
    fn test_deserialize_page() {
        assert_de_tokens(
            &Page {
                path: "test".into(),
                spread: Some(Spread::Center),
            },
            &[
                Token::Map { len: Some(3) },
                Token::Str("path"),
                Token::Str("test"),
                Token::Str("spread"),
                Token::Str("center"),
                Token::MapEnd,
            ],
        );
        assert_de_tokens(
            &Page {
                path: "test".into(),
                spread: None,
            },
            &[Token::String("test")],
        );

        assert_de_tokens_error::<Page>(
            &[Token::Bool(false)],
            "invalid type: boolean `false`, expected a map or a string",
        );
        assert_de_tokens_error::<Page>(
            &[Token::Map { len: Some(1) }, Token::I32(0)],
            "invalid type: integer `0`, expected an identifier",
        );
        assert_de_tokens_error::<Page>(
            &[Token::Map { len: None }, Token::Str("hoge")],
            "unknown field `hoge`, expected `path` or `spread`",
        );
    }

    #[test]
    fn test_serde_spread() {
        assert_tokens(&Spread::Center, &[Token::Str("center")]);

        assert_de_tokens_error::<Spread>(
            &[Token::Str("hoge")],
            "unknown variant `hoge`, expected one of `left`, `right`, `center`",
        );
    }

    #[test]
    fn test_spread_next() {
        assert_eq!(Spread::Left.next(), Some(Spread::Right));
        assert_eq!(Spread::Right.next(), Some(Spread::Left));
        assert_eq!(Spread::Center.next(), Some(Spread::Right));
    }
}
