use std::{collections::HashMap, io::Read};

use crate::error::ReadError;


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

    pub fn read(reader: &mut impl Read) -> Result<Self, ReadError> {
        todo!()
    }
}
