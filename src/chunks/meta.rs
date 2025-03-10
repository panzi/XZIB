use std::io::Read;

use crate::{error::ReadError, Date};

#[derive(Debug)]
pub struct Meta {
    title: String,
    created_at: Date,
    author: Vec<String>,
    license: Vec<String>,
    links: Vec<String>,
    comment: String,
}

impl Meta {
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

    pub fn read(reader: &mut impl Read) -> Result<Self, ReadError> {
        todo!()
    }
}
