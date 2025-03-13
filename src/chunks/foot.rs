use crate::error::{ReadError, ReadErrorKind};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Checksum {
    Crc32(u32),
    Sha1  (Box<[u8; 20]>),
    Sha224(Box<[u8; 28]>),
    Sha256(Box<[u8; 32]>),
    Sha384(Box<[u8; 48]>),
    Sha512(Box<[u8; 64]>),
}

impl Checksum {
    #[inline]
    pub fn from_bytes(checksum_type: ChecksumType, bytes: &[u8]) -> Option<Self> {
        match checksum_type {
            ChecksumType::Crc32 => {
                Some(Checksum::Crc32(u32::from_le_bytes(*bytes[1..].first_chunk()?)))
            }
            ChecksumType::Sha1 => {
                Some(Checksum::Sha1(Box::new(*bytes[1..].first_chunk()?)))
            }
            ChecksumType::Sha224 => {
                Some(Checksum::Sha224(Box::new(*bytes[1..].first_chunk()?)))
            }
            ChecksumType::Sha256 => {
                Some(Checksum::Sha256(Box::new(*bytes[1..].first_chunk()?)))
            }
            ChecksumType::Sha384 => {
                Some(Checksum::Sha384(Box::new(*bytes[1..].first_chunk()?)))
            }
            ChecksumType::Sha512 => {
                Some(Checksum::Sha512(Box::new(*bytes[1..].first_chunk()?)))
            }
        }
    }

    #[inline]
    pub fn checksum_type(&self) -> ChecksumType {
        match self {
            Self::Crc32(_)  => ChecksumType::Crc32,
            Self::Sha1(_)   => ChecksumType::Sha1,
            Self::Sha224(_) => ChecksumType::Sha224,
            Self::Sha256(_) => ChecksumType::Sha256,
            Self::Sha384(_) => ChecksumType::Sha384,
            Self::Sha512(_) => ChecksumType::Sha512,
        }
    }

    #[inline]
    pub fn byte_size(&self) -> usize {
        match self {
            Self::Crc32(_)     => 4,
            Self::Sha1(data)   => data.len(),
            Self::Sha224(data) => data.len(),
            Self::Sha256(data) => data.len(),
            Self::Sha384(data) => data.len(),
            Self::Sha512(data) => data.len(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ChecksumType {
    Crc32  = 1,
    Sha1   = 2,
    Sha224 = 3,
    Sha256 = 4,
    Sha384 = 5,
    Sha512 = 6,
}

impl ChecksumType {
    const CRC32:  u8 = ChecksumType::Crc32  as u8;
    const SHA1:   u8 = ChecksumType::Sha1   as u8;
    const SHA224: u8 = ChecksumType::Sha224 as u8;
    const SHA256: u8 = ChecksumType::Sha256 as u8;
    const SHA384: u8 = ChecksumType::Sha384 as u8;
    const SHA512: u8 = ChecksumType::Sha512 as u8;

    #[inline]
    pub fn new(checksum_type: u8) -> Option<Self> {
        match checksum_type {
            Self::CRC32  => Some(Self::Crc32),
            Self::SHA1   => Some(Self::Sha1),
            Self::SHA224 => Some(Self::Sha224),
            Self::SHA256 => Some(Self::Sha256),
            Self::SHA384 => Some(Self::Sha384),
            Self::SHA512 => Some(Self::Sha512),
            _ => None,
        }
    }
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
        let checksum_type = bytes[0];
        let Some(checksum_type) = ChecksumType::new(checksum_type) else {
            return Err(ReadError::with_message(
                ReadErrorKind::BrokenFile,
                format!("illegal checksum type: {checksum_type}")));
        };

        let checksum = match checksum_type {
            ChecksumType::Crc32 => {
                if let Some(chunk) = bytes[1..].first_chunk() {
                    Some(Checksum::Crc32(u32::from_le_bytes(*chunk)))
                } else { None }
            }
            ChecksumType::Sha1 => {
                if let Some(chunk) = bytes[1..].first_chunk() {
                    Some(Checksum::Sha1(Box::new(*chunk)))
                } else { None }
            }
            _ => todo!()
        };

        let Some(checksum) = checksum else {
            return Err(ReadError::with_message(
                ReadErrorKind::BrokenFile,
                "truncated checksum"));
        };

        Ok(Self { checksum })
    }
}
