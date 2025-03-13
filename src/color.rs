use std::ops::{Add, AddAssign, Mul, MulAssign, Sub, SubAssign, BitAnd, BitAndAssign, BitOr, BitOrAssign, Shl, ShlAssign, Shr, ShrAssign};

use crate::error::{ReadError, ReadErrorKind};


pub trait ChannelValue
where Self: Sized,
      Self: Add,
      Self: AddAssign,
      Self: Sub,
      Self: SubAssign,
      Self: Mul,
      Self: MulAssign,
      Self: std::fmt::Display,
      Self: std::fmt::Debug,
      Self: std::default::Default,
      Self: Clone,
      Self: Copy,
{
    const ZERO: Self;
    const ONE: Self;
    const MAX_VALUE: Self;
    const SIZE: usize = std::mem::size_of::<Self>();

    fn from_bytes(bytes: &[u8]) -> Option<Self>;
}

pub trait IntChannelValue
where Self: ChannelValue,
      Self: BitAnd,
      Self: BitAndAssign,
      Self: BitOr,
      Self: BitOrAssign,
      Self: Shl,
      Self: ShlAssign,
      Self: Shl<usize, Output = Self>,
      Self: ShlAssign<usize>,
      Self: Shr,
      Self: ShrAssign,
      Self: From<u8>
{}

impl ChannelValue for u8 {
    const ZERO: Self = 0;
    const ONE: Self = 1;
    const MAX_VALUE: Self = 0xFF;

    #[inline]
    fn from_bytes(bytes: &[u8]) -> Option<Self> {
        let Some(head) = bytes.first_chunk::<1>() else {
            return None;
        };
        Some(head[0])
    }
}

impl ChannelValue for u16 {
    const ZERO: Self = 0;
    const ONE: Self = 1;
    const MAX_VALUE: Self = 0xFFFF;

    #[inline]
    fn from_bytes(bytes: &[u8]) -> Option<Self> {
        let Some(head) = bytes.first_chunk::<2>() else {
            return None;
        };
        Some(Self::from_le_bytes(*head))
    }
}

impl ChannelValue for u32 {
    const ZERO: Self = 0;
    const ONE: Self = 1;
    const MAX_VALUE: Self = 0xFFFF_FFFF;

    #[inline]
    fn from_bytes(bytes: &[u8]) -> Option<Self> {
        let Some(head) = bytes.first_chunk::<4>() else {
            return None;
        };
        Some(Self::from_le_bytes(*head))
    }
}

impl ChannelValue for u64 {
    const ZERO: Self = 0;
    const ONE: Self = 1;
    const MAX_VALUE: Self = 0xFFFF_FFFF_FFFF_FFFF;

    #[inline]
    fn from_bytes(bytes: &[u8]) -> Option<Self> {
        let Some(head) = bytes.first_chunk::<8>() else {
            return None;
        };
        Some(Self::from_le_bytes(*head))
    }
}

impl ChannelValue for u128 {
    const ZERO: Self = 0;
    const ONE: Self = 1;
    const MAX_VALUE: Self = 0xFFFF_FFFF_FFFF_FFFF_FFFF_FFFF_FFFF_FFFF;

    #[inline]
    fn from_bytes(bytes: &[u8]) -> Option<Self> {
        let Some(head) = bytes.first_chunk::<16>() else {
            return None;
        };
        Some(Self::from_le_bytes(*head))
    }
}

impl IntChannelValue for u8 {}
impl IntChannelValue for u16 {}
impl IntChannelValue for u32 {}
impl IntChannelValue for u64 {}
impl IntChannelValue for u128 {}

// TODO: how to #[cfg()] check this?
//impl ChannelValue for f16 {}
//impl ChannelValue for f128 {}

impl ChannelValue for f32 {
    const ZERO: Self = 0.0;
    const ONE: Self = 1.0;
    const MAX_VALUE: Self = 1.0;

    #[inline]
    fn from_bytes(bytes: &[u8]) -> Option<Self> {
        let Some(head) = bytes.first_chunk::<4>() else {
            return None;
        };
        Some(Self::from_le_bytes(*head))
    }
}

impl ChannelValue for f64 {
    const ZERO: Self = 0.0;
    const ONE: Self = 1.0;
    const MAX_VALUE: Self = 1.0;

    #[inline]
    fn from_bytes(bytes: &[u8]) -> Option<Self> {
        let Some(head) = bytes.first_chunk::<8>() else {
            return None;
        };
        Some(Self::from_le_bytes(*head))
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
#[repr(transparent)]
pub struct Rgb<C: ChannelValue>(pub [C; 3]);

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
#[repr(transparent)]
pub struct Rgba<C: ChannelValue>(pub [C; 4]);

pub trait Color<C: ChannelValue>: std::fmt::Debug + Sized {
    const CHANNELS: usize;

    fn to_rgb(&self) -> Rgb<C>;
    fn to_rgba(&self) -> Rgba<C>;
    fn from_bytes(bytes: &[u8]) -> Option<Self>;
}

impl<C: ChannelValue> Color<C> for Rgb<C> {
    const CHANNELS: usize = 3;

    #[inline]
    fn to_rgb(&self) -> Rgb<C> {
        *self
    }

    #[inline]
    fn to_rgba(&self) -> Rgba<C> {
        let Rgb([r, g, b]) = *self;
        Rgba([r, g, b, C::MAX_VALUE])
    }

    fn from_bytes(mut bytes: &[u8]) -> Option<Self> {
        let Some(r) = C::from_bytes(bytes) else {
            return None;
        };
        bytes = &bytes[C::SIZE..];

        let Some(g) = C::from_bytes(bytes) else {
            return None;
        };
        bytes = &bytes[C::SIZE..];

        let Some(b) = C::from_bytes(bytes) else {
            return None;
        };

        Some(Rgb([r, g, b]))
    }
}

impl<C: ChannelValue> Color<C> for Rgba<C> {
    const CHANNELS: usize = 4;

    #[inline]
    fn to_rgb(&self) -> Rgb<C> {
        let Rgba([r, g, b, _]) = *self;
        Rgb([r, g, b])
    }

    #[inline]
    fn to_rgba(&self) -> Rgba<C> {
        *self
    }

    fn from_bytes(mut bytes: &[u8]) -> Option<Self> {
        let Some(r) = C::from_bytes(bytes) else {
            return None;
        };
        bytes = &bytes[C::SIZE..];

        let Some(g) = C::from_bytes(bytes) else {
            return None;
        };
        bytes = &bytes[C::SIZE..];

        let Some(b) = C::from_bytes(bytes) else {
            return None;
        };
        bytes = &bytes[C::SIZE..];

        let Some(a) = C::from_bytes(bytes) else {
            return None;
        };

        Some(Rgba([r, g, b, a]))
    }
}

impl<C: ChannelValue> Color<C> for C {
    const CHANNELS: usize = 1;

    #[inline]
    fn to_rgb(&self) -> Rgb<C> {
        Rgb([*self, *self, *self])
    }

    #[inline]
    fn to_rgba(&self) -> Rgba<C> {
        Rgba([*self, *self, *self, C::MAX_VALUE])
    }

    #[inline]
    fn from_bytes(bytes: &[u8]) -> Option<Self> {
        C::from_bytes(bytes)
    }
}

pub trait ChannelValueFamily {
    type Data<T> where T: ChannelValue;
}

#[derive(Debug)]
pub enum ChannelVariant<T: ChannelValueFamily> {
    U8(T::Data<u8>),
    U16(T::Data<u16>),
    U32(T::Data<u32>),
    U64(T::Data<u64>),
    U128(T::Data<u128>),
    F32(T::Data<f32>),
    F64(T::Data<f64>),
}

pub trait ColorFamily<C: ChannelValue> {
    type Data<T> where T: Color<C>;
}

#[derive(Debug)]
pub enum ColorVariant<C: ChannelValue, T: ColorFamily<C>> {
    L(T::Data<C>),
    Rgb(T::Data<Rgb<C>>),
    Rgba(T::Data<Rgba<C>>),
}

#[derive(Debug)]
pub struct ColorVecData;

#[derive(Debug)]
pub struct ColorVecDataInner;

impl ChannelValueFamily for ColorVecData {
    type Data<T> = ColorVariant<T, ColorVecDataInner> where T: ChannelValue;
}

impl<C: ChannelValue> ColorFamily<C> for ColorVecDataInner {
    type Data<T> = Vec<T> where T: Color<C>;
}

pub type ColorList = ChannelVariant<ColorVecData>;

pub fn read_colors_into<Color, ChannelValue>(mut bytes: &[u8], colors: &mut Vec<Color>)
where Color: crate::color::Color<ChannelValue>,
      ChannelValue: crate::color::ChannelValue
{
    let color_size = Color::CHANNELS * ChannelValue::SIZE;
    let color_count = bytes.len() / color_size;
    colors.reserve(color_count);

    while let Some(color) = Color::from_bytes(bytes) {
        colors.push(color);
        bytes = &bytes[color_size..];
    }
}

#[inline]
pub fn read_colors<Color, ChannelValue>(bytes: &[u8]) -> Vec<Color>
where Color: crate::color::Color<ChannelValue>,
      ChannelValue: crate::color::ChannelValue
{
    let mut colors = Vec::new();
    read_colors_into(bytes, &mut colors);
    colors
}

#[inline]
pub fn read_colors_variant_inner<C: ChannelValue>(bytes: &[u8], channels: u8) -> Result<ColorVariant<C, ColorVecDataInner>, ReadError> {
    match channels {
        1 => Ok(ColorVariant::L(read_colors(bytes))),
        3 => Ok(ColorVariant::Rgb(read_colors(bytes))),
        4 => Ok(ColorVariant::Rgba(read_colors(bytes))),
        _ => Err(ReadError::with_message(
            ReadErrorKind::BrokenFile,
            format!("illegal number of channels: {channels}")))
    }
}

#[inline]
fn get_bit(bytes: &[u8], bit_index: usize) -> u8 {
    let byte_index = bit_index / 8;
    let bit = bit_index % 8;
    let byte = bytes[byte_index];

    (byte >> bit) & 1
}

#[inline]
fn get_nibble(bytes: &[u8], nibble_index: usize) -> u8 {
    let byte_index = nibble_index >> 1;
    let nibble = nibble_index & 1;
    let byte = bytes[byte_index];

    (byte >> (nibble * 4)) & 0xF
}

pub fn read_colors_variant(bytes: &[u8], is_float: bool, depth: u8, channels: u8) -> Result<ColorList, ReadError> {
    match depth {
        1 if !is_float => {
            match channels {
                1 => {
                    let mut colors = Vec::with_capacity(bytes.len() * 8);
                    for byte in bytes {
                        colors.push(((byte >> 0) & 1) * 255u8);
                        colors.push(((byte >> 1) & 1) * 255u8);
                        colors.push(((byte >> 2) & 1) * 255u8);
                        colors.push(((byte >> 3) & 1) * 255u8);
                        colors.push(((byte >> 4) & 1) * 255u8);
                        colors.push(((byte >> 5) & 1) * 255u8);
                        colors.push(((byte >> 6) & 1) * 255u8);
                        colors.push(((byte >> 7) & 1) * 255u8);
                    }
                    Ok(ChannelVariant::U8(ColorVariant::L(colors)))
                },
                3 => {
                    let color_count = (bytes.len() * 8) / 3;
                    let mut colors = Vec::with_capacity(color_count);
                    for offset in (0..(color_count * 3)).step_by(3) {
                        let r = get_bit(bytes, offset + 0) * 255;
                        let g = get_bit(bytes, offset + 1) * 255;
                        let b = get_bit(bytes, offset + 2) * 255;
                        colors.push(Rgb([r, g, b]));
                    }
                    Ok(ChannelVariant::U8(ColorVariant::Rgb(colors)))
                },
                4 => {
                    let color_count = (bytes.len() * 8) / 4;
                    let mut colors = Vec::with_capacity(color_count);
                    for offset in (0..(color_count * 4)).step_by(4) {
                        let r = get_bit(bytes, offset + 0) * 255;
                        let g = get_bit(bytes, offset + 1) * 255;
                        let b = get_bit(bytes, offset + 2) * 255;
                        let a = get_bit(bytes, offset + 3) * 255;
                        colors.push(Rgba([r, g, b, a]));
                    }
                    Ok(ChannelVariant::U8(ColorVariant::Rgba(colors)))
                },
                _ => Err(ReadError::with_message(ReadErrorKind::BrokenFile, format!("illegal number of channels: {channels}")))
            }
        }
        4 if !is_float => {
            // TODO: propper color mapping
            match channels {
                1 => {
                    let color_count = bytes.len() * 2;
                    let mut colors = Vec::with_capacity(color_count);
                    for offset in 0..color_count {
                        let v = get_nibble(bytes, offset) * 17;
                        colors.push(v);
                    }
                    Ok(ChannelVariant::U8(ColorVariant::L(colors)))
                },
                3 => {
                    let color_count = (bytes.len() * 2) / 3;
                    let mut colors = Vec::with_capacity(color_count);
                    for offset in (0..(color_count * 3)).step_by(3) {
                        let r = get_nibble(bytes, offset + 0) * 17;
                        let g = get_nibble(bytes, offset + 1) * 17;
                        let b = get_nibble(bytes, offset + 2) * 17;
                        colors.push(Rgb([r, g, b]));
                    }
                    Ok(ChannelVariant::U8(ColorVariant::Rgb(colors)))
                },
                4 => {
                    let color_count = (bytes.len() * 2) / 4;
                    let mut colors = Vec::with_capacity(color_count);
                    for offset in (0..(color_count * 4)).step_by(4) {
                        let r = get_nibble(bytes, offset + 0) * 17;
                        let g = get_nibble(bytes, offset + 1) * 17;
                        let b = get_nibble(bytes, offset + 2) * 17;
                        let a = get_nibble(bytes, offset + 3) * 17;
                        colors.push(Rgba([r, g, b, a]));
                    }
                    Ok(ChannelVariant::U8(ColorVariant::Rgba(colors)))
                },
                _ => Err(ReadError::with_message(ReadErrorKind::BrokenFile, format!("illegal number of channels: {channels}")))
            }
        }
        8 if !is_float => {
            Ok(ChannelVariant::U8(read_colors_variant_inner(bytes, channels)?))
        }
        16 if !is_float => {
            Ok(ChannelVariant::U16(read_colors_variant_inner(bytes, channels)?))
        }
        32 => {
            if is_float {
                Ok(ChannelVariant::F32(read_colors_variant_inner(bytes, channels)?))
            } else {
                Ok(ChannelVariant::U32(read_colors_variant_inner(bytes, channels)?))
            }
        }
        64 => {
            if is_float {
                Ok(ChannelVariant::F64(read_colors_variant_inner(bytes, channels)?))
            } else {
                Ok(ChannelVariant::U64(read_colors_variant_inner(bytes, channels)?))
            }
        }
        128 if !is_float => {
            Ok(ChannelVariant::U128(read_colors_variant_inner(bytes, channels)?))
        }
        _ => {
            return Err(ReadError::with_message(
                ReadErrorKind::BrokenFile,
                format!("unsupported color format: {} {depth}", if is_float { "float" } else { "int" })
            ));
        }
    }
}
