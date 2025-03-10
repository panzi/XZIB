use std::io::Read;

use crate::{color::ColorList, error::ReadError, format::Format, Head};

#[derive(Debug)]
pub struct Body {
    data: ColorList
}

impl Body {
    #[inline]
    pub fn new(format: Format) -> Self {
        Self { data: format.make_color_list() }
    }

    #[inline]
    pub fn data(&self) -> &ColorList {
        &self.data
    }

    #[inline]
    pub fn data_mut(&mut self) -> &mut ColorList {
        &mut self.data
    }

    pub fn read(reader: &mut impl Read, head: &Head) -> Result<Self, ReadError> {
        todo!()
    }
}
