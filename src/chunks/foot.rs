use crate::error::ReadError;

#[derive(Debug)]
pub enum Checksum {
    Crc32(u32),
    Sha1  (Box<[u8; 20]>),
    Sha224(Box<[u8; 28]>),
    Sha256(Box<[u8; 32]>),
    Sha384(Box<[u8; 48]>),
    Sha512(Box<[u8; 64]>),
}

#[derive(Debug)]
pub struct Foot {
    checksum: Checksum,
}

impl Foot {
    #[inline]
    pub fn checksum(&self) -> &Checksum {
        &self.checksum
    }

    #[inline]
    pub fn checksum_mut(&mut self) -> &mut Checksum {
        &mut self.checksum
    }

    pub fn read(bytes: &[u8]) -> Result<Self, ReadError> {
        todo!()
    }
}
