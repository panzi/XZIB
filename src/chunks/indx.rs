use std::io::Write;

use crate::{color::{read_colors_variant, ColorList}, error::{ReadError, ReadErrorKind}, Head};

use super::ChunkWrite;

#[derive(Debug)]
pub struct Indx {
    colors: ColorList
}

impl Indx {
    pub const FOURCC: [u8; 4] = *b"INDX";

    #[inline]
    pub fn colors(&self) -> &ColorList {
        &self.colors
    }

    #[inline]
    pub fn colors_mut(&mut self) -> &mut ColorList {
        &mut self.colors
    }

    pub fn read(data: &[u8], head: &Head) -> Result<Self, ReadError> {
        let index_planes = head.index_planes();
        if index_planes == 0 {
            return Err(ReadError::with_message(
                ReadErrorKind::BrokenFile,
                format!("indx chunk found, but index_planes == 0")
            ));
        }

        let colors = read_colors_variant(data, head.is_float(), head.index_planes(), head.channels())?;

        Ok(Self { colors })
    }

    pub fn write(&self, head: &Head, writer: &mut impl Write) -> std::io::Result<()> {
        todo!()
    }
}

impl ChunkWrite for Indx {
    const FOURCC: [u8; 4] = Self::FOURCC;

    #[inline]
    fn write(&self, head: &Head, writer: &mut impl Write) -> std::io::Result<()> {
        self.write(head, writer)
    }
}
