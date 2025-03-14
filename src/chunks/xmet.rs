use core::str;
use std::{collections::HashMap, io::Write};

use crate::{error::{ReadError, ReadErrorKind, WriteError}, Head};

use super::ChunkWrite;


#[derive(Debug)]
pub struct Xmet {
    data: HashMap<String, Vec<String>>,
}

impl Xmet {
    pub const FOURCC: [u8; 4] = *b"XMET";

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

    pub fn write(&self, writer: &mut impl Write) -> Result<(), WriteError> {
        for (key, values) in &self.data {
            for value in values {
                writer.write_all(key.as_bytes())?;
                writer.write_all(&[0])?;

                writer.write_all(value.as_bytes())?;
                writer.write_all(&[0])?;
            }
        }
        Ok(())
    }
}

impl ChunkWrite for Xmet {
    const FOURCC: [u8; 4] = Self::FOURCC;

    #[inline]
    fn write(&self, _head: &Head, writer: &mut impl Write) -> Result<(), WriteError> {
        self.write(writer)
    }
}
