pub mod chunks;
pub mod color;
pub mod format;
pub mod error;
pub mod io;

use std::{borrow::Cow, io::{Read, Seek, Write}};

use chunks::{Body, ChunkWrite, Foot, Indx, Meta, Xmet};
use color::{apply_palette_variant, ChannelVariant, ColorList, ColorVariant};
use error::{IllegalDate, InvalidParams, ReadError, ReadErrorKind, WriteError, WriteErrorKind};
use flate2::{bufread::ZlibDecoder, write::ZlibEncoder, Compression};
use format::{ChannelValueType, ColorType, Format, NumberType};
use io::{read_fourcc, read_u32, read_u64, read_u8};

#[derive(Debug)]
pub struct Head {
    flags: u8,
    channels: u8,
    planes: u8,
    index_planes: u8,
    width: u32,
    height: u32,
}

#[inline]
pub fn is_valid_planes(number_type: NumberType, interleaved: bool, planes: u8) -> bool {
    if number_type.is_float() {
        return matches!(planes, 32 | 64);
    }

    if interleaved {
        return planes > 0 && planes <= 128;
    }

    return matches!(planes, 1 | 4 | 8 | 16 | 32 | 64 | 128);
}

impl Head {
    #[inline]
    pub fn new(number_type: NumberType, interleaved: bool, color_type: ColorType, planes: u8, index_planes: u8, width: u32, height: u32) -> Result<Self, InvalidParams> {
        let mut flags = 0;
        if number_type.is_float() {
            flags |= XZIB::FLOAT;
        }

        if interleaved {
            flags |= XZIB::INTERLEAVED;
        }

        if !is_valid_planes(number_type, interleaved, planes) {
            return Err(InvalidParams::with_message(
                format!("invalid planes for {} {}: {}",
                    if interleaved { "interleaved" } else { "non-interleaved" },
                    number_type, planes
                )
            ));
        }

        if index_planes != 0 && !is_valid_planes(number_type, false, index_planes) {
            return Err(InvalidParams::with_message(
                format!("invalid index planes for {}: {}",
                    number_type, planes
                )
            ));
        }

        if width == 0 || height == 0 {
            return Err(InvalidParams::with_message(
                format!("width and height must be non-zero, but was: {width} x {height}")));
        }

        Ok(Self {
            flags,
            channels: color_type.channels(),
            planes,
            index_planes,
            width,
            height,
        })
    }

    #[inline]
    pub fn is_interleaved(&self) -> bool {
        self.flags & XZIB::INTERLEAVED != 0
    }

    #[inline]
    pub fn is_float(&self) -> bool {
        self.flags & XZIB::FLOAT != 0
    }

    #[inline]
    pub fn is_integer(&self) -> bool {
        self.flags & XZIB::FLOAT == 0
    }

    #[inline]
    pub fn is_indexed(&self) -> bool {
        self.index_planes > 0
    }

    #[inline]
    pub fn number_type(&self) -> NumberType {
        if self.is_float() {
            NumberType::Float
        } else {
            NumberType::Integer
        }
    }

    #[inline]
    pub fn channel_value_type(&self) -> Result<ChannelValueType, InvalidParams> {
        ChannelValueType::from_planes(self.number_type(), self.planes)
    }

    #[inline]
    pub fn index_channel_value_type(&self) -> Result<Option<ChannelValueType>, InvalidParams> {
        if self.index_planes == 0 {
            return Ok(None);
        }
        ChannelValueType::from_planes(self.number_type(), self.index_planes).map(Some)
    }

    #[inline]
    pub fn color_type(&self) -> Result<ColorType, InvalidParams> {
        ColorType::from_channels(self.channels)
    }

    #[inline]
    pub fn format(&self) -> Result<Format, InvalidParams> {
        let channel_value_type = self.channel_value_type()?;
        let color_type = self.color_type()?;
        Ok(Format(channel_value_type, color_type))
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

    pub fn write(&self, writer: &mut impl Write) -> std::io::Result<()> {
        writer.write_all(&XZIB::FOURCC)?;

        writer.write_all(&[
            self.flags,
            self.channels,
            self.planes,
            self.index_planes,
        ])?;

        writer.write_all(&self.width.to_le_bytes())?;
        writer.write_all(&self.height.to_le_bytes())?;

        Ok(())
    }
}

#[derive(Debug)]
pub struct XZIB {
    head: Head,

    indx: Option<Indx>,
    meta: Option<Meta>,
    xmet: Option<Xmet>,
    body: Option<Body>,
    foot: Option<Foot>,
}

impl XZIB {
    pub const INTERLEAVED: u8 = 1;
    pub const FLOAT: u8 = 2;
    pub const FOURCC: [u8; 4] = *b"XZIB";

    #[inline]
    pub fn new(head: Head) -> Self {
        Self {
            head,
            indx: None,
            meta: None,
            xmet: None,
            body: None,
            foot: None,
        }
    }

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

    #[inline]
    pub fn indx_mut(&mut self) -> &mut Option<Indx> {
        &mut self.indx
    }

    #[inline]
    pub fn meta_mut(&mut self) -> &mut Option<Meta> {
        &mut self.meta
    }

    #[inline]
    pub fn xmet_mut(&mut self) -> &mut Option<Xmet> {
        &mut self.xmet
    }

    #[inline]
    pub fn body_mut(&mut self) -> &mut Option<Body> {
        &mut self.body
    }

    #[inline]
    pub fn foot_mut(&mut self) -> &mut Option<Foot> {
        &mut self.foot
    }

    pub fn image_buffer(&self) -> Option<Cow<ColorList>> {
        let Some(body) = &self.body else {
            return None;
        };

        let data = body.data();

        if let Some(indx) = &self.indx {
            match data {
                ChannelVariant::U8(data) => {
                    if let ColorVariant::L(data) = data {
                        return Some(Cow::Owned(apply_palette_variant(data, indx.colors())))
                    }
                }
                ChannelVariant::U16(data) => {
                    if let ColorVariant::L(data) = data {
                        return Some(Cow::Owned(apply_palette_variant(data, indx.colors())))
                    }
                }
                ChannelVariant::U32(data) => {
                    if let ColorVariant::L(data) = data {
                        return Some(Cow::Owned(apply_palette_variant(data, indx.colors())))
                    }
                }
                ChannelVariant::U64(data) => {
                    if let ColorVariant::L(data) = data {
                        return Some(Cow::Owned(apply_palette_variant(data, indx.colors())))
                    }
                }
                ChannelVariant::U128(data) => {
                    if let ColorVariant::L(data) = data {
                        return Some(Cow::Owned(apply_palette_variant(data, indx.colors())))
                    }
                }
                _ => {}
            }
        }

        Some(Cow::Borrowed(data))
    }

    pub fn read<R>(reader: &mut R) -> Result<Self, ReadError>
    where R: Read + Seek {
        let head = Head::read(reader)?;

        if head.channels() == 0 {
            return Err(ReadError::with_message(
                ReadErrorKind::BrokenFile,
                format!("channels == 0")
            ));
        }

        if head.planes() == 0 {
            return Err(ReadError::with_message(
                ReadErrorKind::BrokenFile,
                format!("planes == 0")
            ));
        }

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

            match fourcc {
                Indx::FOURCC => {
                    indx = Some(Indx::read(chunk_data, &head)?);
                }
                Meta::FOURCC => {
                    meta = Some(Meta::read(chunk_data)?);
                }
                Xmet::FOURCC => {
                    xmet = Some(Xmet::read(chunk_data)?);
                }
                Body::FOURCC => {
                    body = Some(Body::read(chunk_data, &head)?);
                }
                Foot::FOURCC => {
                    foot = Some(Foot::read(chunk_data)?);
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

    pub fn write(&self, writer: &mut impl Write, compression: u32) -> Result<(), WriteError> {
        self.head.write(writer)?;

        let mut buf = Vec::new();

        let compression = Compression::new(compression);

        if let Some(indx) = &self.indx {
            self.write_chunk(&mut buf, writer, indx, compression)?;
        }

        if let Some(meta) = &self.meta {
            self.write_chunk(&mut buf, writer, meta, compression)?;
        }

        if let Some(xmet) = &self.xmet {
            self.write_chunk(&mut buf, writer, xmet, compression)?;
        }

        if let Some(body) = &self.body {
            if self.indx.is_some() && body.data().color_type() != ColorType::L {
                return Err(WriteError::with_message(
                    WriteErrorKind::InvalidParams,
                    format!("using an index the BODY chunk must be of type L, but was: {}",
                        body.data().color_type())));
            }

            self.write_chunk(&mut buf, writer, body, compression)?;
        }

        if let Some(foot) = &self.foot {
            // TODO: pipe all writes through a writer that calculates the checksum!
            self.write_chunk(&mut buf, writer, foot, compression)?;
        }

        Ok(())
    }

    fn write_chunk(&self, mut buf: &mut Vec<u8>, writer: &mut impl Write, chunk: &impl ChunkWrite, compression: Compression) -> Result<(), WriteError> {
        buf.clear();
        let mut fourcc = Indx::FOURCC;
        if compression.level() > 0 {
            let mut encoder = ZlibEncoder::new(&mut buf, compression);
            chunk.write(&self.head, &mut encoder)?;
            encoder.finish()?;
            fourcc[1] = fourcc[1].to_ascii_lowercase();
        } else {
            chunk.write(&self.head, buf)?;
        }

        if buf.len() <= u32::MAX as usize {
            fourcc[0] = fourcc[0].to_ascii_lowercase();
            writer.write_all(&fourcc)?;
            writer.write_all(&(buf.len() as u32).to_le_bytes())?;
        } else {
            writer.write_all(&fourcc)?;
            writer.write_all(&(buf.len() as u64).to_le_bytes())?;
        }

        writer.write_all(&buf)?;

        Ok(())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Default)]
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

    pub fn parse(value: &str) -> Result<Date, IllegalDate> {
        let mut iter = value.split('-');

        let Some(year) = iter.next() else {
            return Err(IllegalDate::with_message(format!("illegal date: {value:?}")));
        };
        let Some(month) = iter.next() else {
            return Err(IllegalDate::with_message(format!("illegal date: {value:?}")));
        };
        let Some(day) = iter.next() else {
            return Err(IllegalDate::with_message(format!("illegal date: {value:?}")));
        };

        if let Some(_) = iter.next() {
            return Err(IllegalDate::with_message(format!("illegal date: {value:?}")));
        }

        let Ok(year) = year.parse() else {
            return Err(IllegalDate::with_message(format!("illegal date: {value:?}")));
        };
        let Ok(month) = month.parse() else {
            return Err(IllegalDate::with_message(format!("illegal date: {value:?}")));
        };
        let Ok(day) = day.parse() else {
            return Err(IllegalDate::with_message(format!("illegal date: {value:?}")));
        };

        Ok(Self { year, month, day })
    }
}

impl std::fmt::Display for Date {
    #[inline]
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:04}-{:02}-{:02}", self.year, self.month, self.day)
    }
}
