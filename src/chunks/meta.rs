use core::str;
use std::io::Write;

use crate::{error::{IllegalMetaKey, ReadError, ReadErrorKind, WriteError}, Date, Head};

use super::ChunkWrite;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
#[repr(u8)]
pub enum MetaKey {
    Title     = 1,
    CreatedAt = 2,
    Author    = 3,
    License   = 4,
    Links     = 5,
    Comment   = 6,
}

impl TryFrom<u8> for MetaKey {
    type Error = IllegalMetaKey;
    #[inline]
    fn try_from(value: u8) -> Result<Self, Self::Error> {
        if value == MetaKey::Title as u8 {
            Ok(MetaKey::Title)
        } else if value == MetaKey::CreatedAt as u8 {
            Ok(MetaKey::CreatedAt)
        } else if value == MetaKey::Author as u8 {
            Ok(MetaKey::Author)
        } else if value == MetaKey::License as u8 {
            Ok(MetaKey::License)
        } else if value == MetaKey::Links as u8 {
            Ok(MetaKey::Links)
        } else if value == MetaKey::Comment as u8 {
            Ok(MetaKey::Comment)
        } else {
            Err(IllegalMetaKey::with_message(value, format!("illegal meta key: {value}")))
        }
    }
}

#[derive(Debug, Clone)]
pub struct Meta {
    title: String,
    created_at: Date,
    author: Vec<String>,
    license: Vec<String>,
    links: Vec<String>,
    comment: String,
}

impl Meta {
    pub const FOURCC: [u8; 4] = *b"META";

    #[inline]
    pub fn title(&self) -> &str {
        &self.title
    }

    #[inline]
    pub fn created_at(&self) -> &Date {
        &self.created_at
    }

    #[inline]
    pub fn author(&self) -> &[impl AsRef<str>] {
        &self.author
    }

    #[inline]
    pub fn license(&self) -> &[impl AsRef<str>] {
        &self.license
    }

    #[inline]
    pub fn links(&self) -> &[impl AsRef<str>] {
        &self.links
    }

    #[inline]
    pub fn comment(&self) -> &str {
        &self.comment
    }

    #[inline]
    pub fn set_title(&mut self, value: impl Into<String>) {
        self.title = value.into();
    }

    #[inline]
    pub fn set_created_at(&mut self, value: &Date) {
        self.created_at = *value;
    }

    #[inline]
    pub fn title_mut(&mut self) -> &mut String {
        &mut self.title
    }

    #[inline]
    pub fn created_at_mut(&mut self) -> &mut Date {
        &mut self.created_at
    }

    #[inline]
    pub fn author_mut(&mut self) -> &mut Vec<String> {
        &mut self.author
    }

    #[inline]
    pub fn license_mut(&mut self) -> &mut Vec<String> {
        &mut self.license
    }

    #[inline]
    pub fn links_mut(&mut self) -> &mut Vec<String> {
        &mut self.links
    }

    #[inline]
    pub fn comment_mut(&mut self) -> &mut String {
        &mut self.comment
    }

    pub fn read(mut data: &[u8]) -> Result<Self, ReadError> {
        let mut title = String::new();
        let mut created_at = Date::default();
        let mut author = Vec::new();
        let mut license = Vec::new();
        let mut links = Vec::new();
        let mut comment = String::new();

        while !data.is_empty() {
            let Some(&key) = data.first() else {
                break;
            };
            data = &data[1..];

            if key == 0 {
                break;
            }

            let chunk = if let Some(chunk) = data.split(|b| *b == 0).next() {
                chunk
            } else {
                eprintln!("last meta data entry not NUL terminated");
                data
            };
            data = &data[chunk.len() + 1..];

            let value = match str::from_utf8(chunk) {
                Ok(value) => value,
                Err(err) => return Err(ReadError::with_all(
                    ReadErrorKind::BrokenFile,
                    format!("illegal UTF-8 bytes in meta data value: {chunk:?}"),
                    Box::new(err)))
            };
            let Ok(key) = MetaKey::try_from(key) else {
                eprintln!("unknown meta key: {key}");
                continue;
            };

            match key {
                MetaKey::Title => {
                    title.clear();
                    title.push_str(value);
                }
                MetaKey::Author => {
                    author.push(value.to_string());
                }
                MetaKey::Comment => {
                    if !comment.is_empty() {
                        comment.push_str("\n");
                    }
                    comment.push_str(value);
                }
                MetaKey::CreatedAt => {
                    created_at = match Date::parse(value) {
                        Ok(date) => date,
                        Err(err) => {
                            eprint!("illegal created_at meta data value: {err}");
                            continue;
                        }
                    };
                }
                MetaKey::License => {
                    license.push(value.to_string());
                }
                MetaKey::Links => {
                    links.push(value.to_string());
                }
            }
        }

        Ok(Self {
            title,
            created_at,
            author,
            comment,
            license,
            links,
        })
    }
    
    pub fn write(&self, writer: &mut impl Write) -> Result<(), WriteError> {
        if !self.title.is_empty() {
            writer.write_all(&[MetaKey::Title as u8])?;
            writer.write_all(self.title.as_bytes())?;
            writer.write_all(&[0])?;
        }

        if !self.created_at.is_null() {
            writer.write_all(&[MetaKey::CreatedAt as u8])?;
            write!(writer, "{}", self.created_at)?;
            writer.write_all(&[0])?;
        }

        for author in &self.author {
            writer.write_all(&[MetaKey::Author as u8])?;
            writer.write_all(author.as_bytes())?;
            writer.write_all(&[0])?;
        }

        if !self.comment.is_empty() {
            writer.write_all(&[MetaKey::Comment as u8])?;
            writer.write_all(self.comment.as_bytes())?;
            writer.write_all(&[0])?;
        }

        for license in &self.license {
            writer.write_all(&[MetaKey::License as u8])?;
            writer.write_all(license.as_bytes())?;
            writer.write_all(&[0])?;
        }

        for links in &self.links {
            writer.write_all(&[MetaKey::Links as u8])?;
            writer.write_all(links.as_bytes())?;
            writer.write_all(&[0])?;
        }

        Ok(())
    }
}

impl ChunkWrite for Meta {
    const FOURCC: [u8; 4] = Self::FOURCC;

    #[inline]
    fn write(&self, _head: &Head, writer: &mut impl Write) -> Result<(), WriteError> {
        self.write(writer)
    }
}
