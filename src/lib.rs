pub mod chunks;
pub mod color;
pub mod builder;
pub mod format;
pub mod error;
pub mod io;

use std::io::{Cursor, Read};

use chunks::{Body, Foot, Indx, Meta, Xmet};
use error::{ReadError, ReadErrorKind};
use flate2::bufread::ZlibDecoder;
use io::{read_fourcc, read_u32, read_u64, read_u8};

#[derive(Debug)]
pub struct Head {
    pub(crate) flags: u8,
    pub(crate) channels: u8,
    pub(crate) planes: u8,
    pub(crate) index_planes: u8,
    pub(crate) width: u32,
    pub(crate) height: u32,
}

impl Head {
    #[inline]
    pub fn is_interleaved(&self) -> bool {
        self.flags & XZIB::INTERLEAVED != 0
    }

    #[inline]
    pub fn is_float(&self) -> bool {
        self.flags & XZIB::FLOAT != 0
    }

    #[inline]
    pub fn flags(&self) -> u8 {
        self.flags
    }

    #[inline]
    pub fn channels(&self) -> u8 {
        self.channels
    }

    #[inline]
    pub fn planes(&self) -> u8 {
        self.planes
    }

    #[inline]
    pub fn index_planes(&self) -> u8 {
        self.index_planes
    }

    #[inline]
    pub fn width(&self) -> u32 {
        self.width
    }

    #[inline]
    pub fn height(&self) -> u32 {
        self.height
    }

    pub fn read(reader: &mut impl Read) -> Result<Self, ReadError> {
        let fourcc = read_fourcc(reader)?;

        if fourcc != XZIB::FOURCC {
            return Err(ReadError::with_message(
                ReadErrorKind::Unsupported,
                format!("unsupported fourcc: {:?}", fourcc)));
        }

        let flags        = read_u8(reader)?;
        let channels     = read_u8(reader)?;
        let planes       = read_u8(reader)?;
        let index_planes = read_u8(reader)?;
        let width        = read_u32(reader)?;
        let height       = read_u32(reader)?;

        Ok(Self {
            flags,
            channels,
            planes,
            index_planes,
            width,
            height,
        })
    }
}

#[derive(Debug)]
pub struct XZIB {
    pub(crate) head: Head,

    pub(crate) indx: Option<Indx>,
    pub(crate) meta: Option<Meta>,
    pub(crate) xmet: Option<Xmet>,
    pub(crate) body: Option<Body>,
    pub(crate) foot: Option<Foot>,
}

impl XZIB {
    pub const INTERLEAVED: u8 = 1;
    pub const FLOAT: u8 = 2;
    pub const FOURCC: [u8; 4] = *b"XZIB";

    #[inline]
    pub fn head(&self) -> &Head {
        &self.head
    }

    #[inline]
    pub fn indx(&self) -> Option<&Indx> {
        self.indx.as_ref()
    }

    #[inline]
    pub fn meta(&self) -> Option<&Meta> {
        self.meta.as_ref()
    }

    #[inline]
    pub fn xmet(&self) -> Option<&Xmet> {
        self.xmet.as_ref()
    }

    #[inline]
    pub fn body(&self) -> Option<&Body> {
        self.body.as_ref()
    }

    #[inline]
    pub fn foot(&self) -> Option<&Foot> {
        self.foot.as_ref()
    }

    pub fn read(reader: &mut impl Read) -> Result<Self, ReadError> {
        let head = Head::read(reader)?;

        let mut indx: Option<Indx> = None;
        let mut meta: Option<Meta> = None;
        let mut xmet: Option<Xmet> = None;
        let mut body: Option<Body> = None;
        let mut foot: Option<Foot> = None;

        let mut buf = Vec::new();
        let mut decompr = Vec::new();

        loop {
            let fourcc = match read_fourcc(reader) {
                Ok(fourcc) => fourcc,
                Err(err) => {
                    if err.kind() == std::io::ErrorKind::UnexpectedEof {
                        break;
                    }
                    return Err(err.into());
                }
            };

            let chunk_size = if fourcc[0].is_ascii_uppercase() {
                read_u64(reader)? as usize
            } else {
                read_u32(reader)? as usize
            };

            buf.resize(chunk_size, 0u8);
            reader.read_exact(&mut buf)?;

            let chunk_data = if fourcc[1].is_ascii_lowercase() {
                decompr.clear();
                let mut decoder = ZlibDecoder::new(&buf[..]);
                decoder.read_to_end(&mut decompr)?;
                &decompr[..]
            } else {
                &buf[..]
            };

            let fourcc = [
                fourcc[0].to_ascii_uppercase(),
                fourcc[1].to_ascii_uppercase(),
                fourcc[2],
                fourcc[3],
            ];

            match &fourcc {
                b"INDX" => {
                    indx = Some(Indx::read(&chunk_data, &head)?);
                }
                b"META" => {
                    meta = Some(Meta::read(&mut Cursor::new(chunk_data))?);
                }
                b"XMET" => {
                    xmet = Some(Xmet::read(&mut Cursor::new(chunk_data))?);
                }
                b"BODY" => {
                    body = Some(Body::read(&mut Cursor::new(chunk_data), &head)?);
                }
                b"FOOT" => {
                    foot = Some(Foot::read(&mut Cursor::new(chunk_data))?);
                }
                _ => {
                    eprintln!("ignored unknown chunk: {:?}", fourcc);
                }
            }
        }

        Ok(Self {
            head,
            indx,
            meta,
            xmet,
            body,
            foot,
        })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Date {
    year: u16,
    month: u8,
    day:   u8,
}

impl Date {
    #[inline]
    pub fn new(year: u16, month: u8, day: u8) -> Self {
        Self { year, month, day }
    }

    #[inline]
    pub fn from_year(year: u16) -> Self {
        Self { year, month: 0, day: 0 }
    }

    #[inline]
    pub fn from_year_and_month(year: u16, month: u8) -> Self {
        Self { year, month, day: 0 }
    }

    #[inline]
    pub fn year(&self) -> u16 {
        self.year
    }

    #[inline]
    pub fn month(&self) -> u8 {
        self.month
    }

    #[inline]
    pub fn day(&self) -> u8 {
        self.day
    }

    #[inline]
    pub fn set_year(&mut self, value: u16) {
        self.year = value;
    }

    #[inline]
    pub fn set_month(&mut self, value: u8) {
        self.month = value;
    }

    #[inline]
    pub fn set_day(&mut self, value: u8) {
        self.day = value;
    }

    #[inline]
    pub fn is_null(&self) -> bool {
        (self.year & self.month as u16 & self.day as u16) == 0
    }
}
