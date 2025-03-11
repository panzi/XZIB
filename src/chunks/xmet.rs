use core::str;
use std::collections::HashMap;

use crate::error::{ReadError, ReadErrorKind};


#[derive(Debug)]
pub struct Xmet {
    data: HashMap<String, Vec<String>>,
}

impl Xmet {
    #[inline]
    pub fn data(&self) -> &HashMap<String, Vec<String>> {
        &self.data
    }

    #[inline]
    pub fn data_mut(&mut self) -> &mut HashMap<String, Vec<String>> {
        &mut self.data
    }

    pub fn read(mut bytes: &[u8]) -> Result<Self, ReadError> {
        let mut data: HashMap<String, Vec<String>> = HashMap::new();

        while !bytes.is_empty() {
            let chunk = if let Some(chunk) = bytes.split(|b| *b == 0).next() {
                chunk
            } else {
                eprintln!("last meta data key not NUL terminated");
                bytes
            };
            bytes = &bytes[chunk.len() + 1..];

            let key = match str::from_utf8(chunk) {
                Ok(key) => key,
                Err(err) => return Err(ReadError::with_all(
                    ReadErrorKind::BrokenFile,
                    format!("illegal UTF-8 bytes in meta data key: {chunk:?}"),
                    Box::new(err)))
            };

            if key.is_empty() {
                break;
            }

            let chunk = if let Some(chunk) = bytes.split(|b| *b == 0).next() {
                chunk
            } else {
                eprintln!("last meta data value not NUL terminated");
                bytes
            };
            bytes = &bytes[chunk.len() + 1..];

            let value = match str::from_utf8(chunk) {
                Ok(value) => value,
                Err(err) => return Err(ReadError::with_all(
                    ReadErrorKind::BrokenFile,
                    format!("illegal UTF-8 bytes in meta data value: {chunk:?}"),
                    Box::new(err)))
            };

            data.entry(key.to_owned())
                .and_modify(|values| values.push(value.to_owned()))
                .or_insert_with(|| vec![value.to_owned()]);
        }

        Ok(Self {
            data
        })
    }
}
