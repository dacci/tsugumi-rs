pub mod ebpaj;

use serde::de;
use serde::{Deserialize, Serialize};
use std::fmt;
use std::path::PathBuf;

#[derive(Deserialize, Serialize)]
pub struct Book {
    pub metadata: Metadata,
    pub cover: PathBuf,
    #[serde(default)]
    pub chapters: Vec<Chapter>,
}

#[derive(Deserialize, Serialize)]
pub struct Metadata {
    pub title: String,
    pub author: String,
}

#[derive(Deserialize, Serialize)]
pub struct Chapter {
    pub name: Option<String>,
    #[serde(default)]
    pub pages: Vec<Page>,
}

#[derive(Debug, PartialEq, Eq, Serialize)]
pub struct Page {
    pub path: PathBuf,
}

impl<'de> de::Deserialize<'de> for Page {
    fn deserialize<D: de::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        enum Field {
            Path,
            _Unknown,
        }

        struct FieldVisitor;

        impl<'de> de::Visitor<'de> for FieldVisitor {
            type Value = Field;

            fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
                f.write_str("an identifier")
            }

            fn visit_str<E: de::Error>(self, v: &str) -> Result<Self::Value, E> {
                match v {
                    "path" => Ok(Field::Path),
                    _ => Ok(Field::_Unknown),
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
                })
            }

            fn visit_string<E: de::Error>(self, v: String) -> Result<Self::Value, E> {
                Ok(Page {
                    path: PathBuf::from(v),
                })
            }

            fn visit_map<A: de::MapAccess<'de>>(self, mut map: A) -> Result<Self::Value, A::Error> {
                let mut path = None;

                while let Some(field) = map.next_key()? {
                    match field {
                        Field::Path => {
                            if path.is_some() {
                                return Err(de::Error::duplicate_field("path"));
                            }
                            path = Some(map.next_value()?);
                        }
                        Field::_Unknown => {
                            map.next_value::<de::IgnoredAny>()?;
                        }
                    }
                }

                let path = path.ok_or_else(|| de::Error::missing_field("path"))?;

                Ok(Page { path })
            }
        }

        deserializer.deserialize_any(Visitor)
    }
}

#[derive(Deserialize, Serialize)]
pub struct Style {
    #[serde(default)]
    pub links: Vec<PathBuf>,
    #[serde(default)]
    pub includes: Vec<PathBuf>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_test::{assert_de_tokens, assert_de_tokens_error, Token};

    #[test]
    fn test_deserialize_page() {
        let page = Page {
            path: "test".into(),
        };

        assert_de_tokens(
            &page,
            &[
                Token::Map { len: Some(1) },
                Token::Str("answer"),
                Token::I32(42),
                Token::Str("path"),
                Token::Str("test"),
                Token::MapEnd,
            ],
        );
        assert_de_tokens(&page, &[Token::String("test")]);

        assert_de_tokens_error::<Page>(
            &[Token::Bool(false)],
            "invalid type: boolean `false`, expected a map or a string",
        );
        assert_de_tokens_error::<Page>(
            &[Token::Map { len: Some(1) }, Token::I32(0)],
            "invalid type: integer `0`, expected an identifier",
        );
    }
}
