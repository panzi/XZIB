use crate::{color::ColorList, error::ReadError, Head};

#[derive(Debug)]
pub struct Indx {
    colors: ColorList
}

impl Indx {
    #[inline]
    pub fn colors(&self) -> &ColorList {
        &self.colors
    }

    #[inline]
    pub fn colors_mut(&mut self) -> &mut ColorList {
        &mut self.colors
    }

    pub fn read(data: &[u8], head: &Head) -> Result<Self, ReadError> {
        todo!()
    }
}
