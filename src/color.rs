use std::{io::Write, ops::{Add, AddAssign, BitAnd, BitAndAssign, BitOr, BitOrAssign, Mul, MulAssign, Shl, ShlAssign, Shr, ShrAssign, Sub, SubAssign}};

use crate::{error::{ReadError, ReadErrorKind, WriteError, WriteErrorKind}, format::{ChannelValueType, ColorType}};


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
    const SIZE: u32 = std::mem::size_of::<Self>() as u32;
    const BITS: u32 = Self::SIZE * 8;

    fn from_bytes(bytes: &[u8]) -> Option<Self>;
    fn write_to(&self, writer: impl Write) -> std::io::Result<()>;
}

pub trait IntChannelValue
where Self: ChannelValue,
      Self: BitAnd<Output = Self>,
      Self: BitAndAssign,
      Self: BitOr<Output = Self>,
      Self: BitOrAssign,
      Self: Shl<Output = Self>,
      Self: ShlAssign,
      Self: Shl<usize, Output = Self>,
      Self: ShlAssign<usize>,
      Self: Shl<u32, Output = Self>,
      Self: ShlAssign<u32>,
      Self: Shr<Output = Self>,
      Self: ShrAssign,
      Self: Shr<usize, Output = Self>,
      Self: ShrAssign<usize>,
      Self: Shr<u32, Output = Self>,
      Self: ShrAssign<u32>,
      Self: From<u8>,
{
    #[inline]
    fn extend(self, planes: u8) -> Self {
        debug_assert!(planes as u32 >= (Self::BITS / 2) && planes as u32 <= Self::BITS);

        let lshift = Self::BITS - planes as u32;
        let rshift = (planes as u32 * 2) - Self::BITS;

        (self << lshift) | (self >> rshift)
    }

    fn least_significant_byte(self) -> u8;
}

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

    #[inline]
    fn write_to(&self, mut writer: impl Write) -> std::io::Result<()> {
        writer.write_all(&self.to_le_bytes())
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

    #[inline]
    fn write_to(&self, mut writer: impl Write) -> std::io::Result<()> {
        writer.write_all(&self.to_le_bytes())
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

    #[inline]
    fn write_to(&self, mut writer: impl Write) -> std::io::Result<()> {
        writer.write_all(&self.to_le_bytes())
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

    #[inline]
    fn write_to(&self, mut writer: impl Write) -> std::io::Result<()> {
        writer.write_all(&self.to_le_bytes())
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

    #[inline]
    fn write_to(&self, mut writer: impl Write) -> std::io::Result<()> {
        writer.write_all(&self.to_le_bytes())
    }
}

impl IntChannelValue for u8 {
    #[inline]
    fn extend(self, planes: u8) -> Self {
        // TODO: branchless?
        match planes {
            1 => {
                self * 255
            }
            2 => {
                self << 6 | self << 4 | self << 2 | self
            }
            3 => {
                self << 5 | self << 2 | self >> 1
            }
            4 => {
                self << 4 | self
            }
            5 => {
                self << 3 | self >> 2
            }
            6 => {
                self << 2 | self >> 4
            }
            7 => {
                self << 1 | self >> 6
            }
            _ => {
                debug_assert!(false, "illegal planes for u8: {planes}");
                self
            }
        }
    }

    #[inline]
    fn least_significant_byte(self) -> u8 {
        self as u8
    }
}

impl IntChannelValue for u16 {
    #[inline]
    fn least_significant_byte(self) -> u8 {
        self as u8
    }
}

impl IntChannelValue for u32 {
    #[inline]
    fn least_significant_byte(self) -> u8 {
        self as u8
    }
}

impl IntChannelValue for u64 {
    #[inline]
    fn least_significant_byte(self) -> u8 {
        self as u8
    }
}

impl IntChannelValue for u128 {
    #[inline]
    fn least_significant_byte(self) -> u8 {
        self as u8
    }
}

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

    #[inline]
    fn write_to(&self, mut writer: impl Write) -> std::io::Result<()> {
        writer.write_all(&self.to_le_bytes())
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

    #[inline]
    fn write_to(&self, mut writer: impl Write) -> std::io::Result<()> {
        writer.write_all(&self.to_le_bytes())
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
#[repr(transparent)]
pub struct La<C: ChannelValue>(pub [C; 2]);

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
#[repr(transparent)]
pub struct Rgb<C: ChannelValue>(pub [C; 3]);

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
#[repr(transparent)]
pub struct Rgba<C: ChannelValue>(pub [C; 4]);

pub trait Color<C: ChannelValue>: std::fmt::Debug + Sized + Default + Clone {
    const CHANNELS: u8;

    fn to_rgb(&self) -> Rgb<C>;
    fn to_rgba(&self) -> Rgba<C>;
    fn from_bytes(bytes: &[u8]) -> Option<Self>;
    fn channels(&self) -> &[C];
    fn channels_mut(&mut self) -> &mut [C];

    #[inline]
    fn write_to(&self, mut writer: impl Write) -> std::io::Result<()> {
        for channel in self.channels() {
            channel.write_to(&mut writer)?;
        }

        Ok(())
    }
}

impl<C: ChannelValue> Color<C> for La<C> {
    const CHANNELS: u8 = 2;

    #[inline]
    fn to_rgb(&self) -> Rgb<C> {
        let &La([l, _]) = self;
        Rgb([l, l, l])
    }

    #[inline]
    fn to_rgba(&self) -> Rgba<C> {
        let &La([l, a]) = self;
        Rgba([l, l, l, a])
    }

    fn from_bytes(mut bytes: &[u8]) -> Option<Self> {
        let Some(l) = C::from_bytes(bytes) else {
            return None;
        };
        bytes = &bytes[C::SIZE as usize..];

        let Some(a) = C::from_bytes(bytes) else {
            return None;
        };
        
        Some(La([l, a]))
    }

    #[inline]
    fn channels(&self) -> &[C] {
        &self.0
    }

    #[inline]
    fn channels_mut(&mut self) -> &mut [C] {
        &mut self.0
    }
}

impl<C: ChannelValue> Color<C> for Rgb<C> {
    const CHANNELS: u8 = 3;

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
        bytes = &bytes[C::SIZE as usize..];

        let Some(g) = C::from_bytes(bytes) else {
            return None;
        };
        bytes = &bytes[C::SIZE as usize..];

        let Some(b) = C::from_bytes(bytes) else {
            return None;
        };

        Some(Rgb([r, g, b]))
    }

    #[inline]
    fn channels(&self) -> &[C] {
        &self.0
    }

    #[inline]
    fn channels_mut(&mut self) -> &mut [C] {
        &mut self.0
    }
}

impl<C: ChannelValue> Color<C> for Rgba<C> {
    const CHANNELS: u8 = 4;

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
        bytes = &bytes[C::SIZE as usize..];

        let Some(g) = C::from_bytes(bytes) else {
            return None;
        };
        bytes = &bytes[C::SIZE as usize..];

        let Some(b) = C::from_bytes(bytes) else {
            return None;
        };
        bytes = &bytes[C::SIZE as usize..];

        let Some(a) = C::from_bytes(bytes) else {
            return None;
        };

        Some(Rgba([r, g, b, a]))
    }

    #[inline]
    fn channels(&self) -> &[C] {
        &self.0
    }

    #[inline]
    fn channels_mut(&mut self) -> &mut [C] {
        &mut self.0
    }
}

impl<C: ChannelValue> Color<C> for C {
    const CHANNELS: u8 = 1;

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

    #[inline]
    fn channels(&self) -> &[C] {
        std::slice::from_ref(self)
    }

    #[inline]
    fn channels_mut(&mut self) -> &mut [C] {
        std::slice::from_mut(self)
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

impl<T: ChannelValueFamily> ChannelVariant<T> {
    #[inline]
    pub fn channel_value_type(&self) -> ChannelValueType {
        match self {
            ChannelVariant::U8(_)   => ChannelValueType::U8,
            ChannelVariant::U16(_)  => ChannelValueType::U16,
            ChannelVariant::U32(_)  => ChannelValueType::U32,
            ChannelVariant::U64(_)  => ChannelValueType::U64,
            ChannelVariant::U128(_) => ChannelValueType::U128,
            ChannelVariant::F32(_)  => ChannelValueType::F32,
            ChannelVariant::F64(_)  => ChannelValueType::F64,
        }
    }
}

impl<T: ChannelValueFamily> Clone for ChannelVariant<T>
where <T as ChannelValueFamily>::Data<u8>: Clone,
      <T as ChannelValueFamily>::Data<u16>: Clone,
      <T as ChannelValueFamily>::Data<u32>: Clone,
      <T as ChannelValueFamily>::Data<u64>: Clone,
      <T as ChannelValueFamily>::Data<u128>: Clone,
      <T as ChannelValueFamily>::Data<f32>: Clone,
      <T as ChannelValueFamily>::Data<f64>: Clone,
{
    #[inline]
    fn clone(&self) -> Self {
        match self {
            ChannelVariant::U8(data)   => ChannelVariant::U8(data.clone()),
            ChannelVariant::U16(data)  => ChannelVariant::U16(data.clone()),
            ChannelVariant::U32(data)  => ChannelVariant::U32(data.clone()),
            ChannelVariant::U64(data)  => ChannelVariant::U64(data.clone()),
            ChannelVariant::U128(data) => ChannelVariant::U128(data.clone()),
            ChannelVariant::F32(data)  => ChannelVariant::F32(data.clone()),
            ChannelVariant::F64(data)  => ChannelVariant::F64(data.clone()),
        }
    }
}

pub trait ColorFamily<C: ChannelValue> {
    type Data<T> where T: Color<C>;
}

#[derive(Debug)]
pub enum ColorVariant<C: ChannelValue, T: ColorFamily<C>> {
    L(T::Data<C>),
    La(T::Data<La<C>>),
    Rgb(T::Data<Rgb<C>>),
    Rgba(T::Data<Rgba<C>>),
}

impl<C: ChannelValue, T: ColorFamily<C>> ColorVariant<C, T> {
    #[inline]
    pub fn color_type(&self) -> ColorType {
        match self {
            Self::L(_)    => ColorType::L,
            Self::La(_)   => ColorType::La,
            Self::Rgb(_)  => ColorType::Rgb,
            Self::Rgba(_) => ColorType::Rgba,
        }
    }
}

impl<C: ChannelValue, T: ColorFamily<C>> Clone for ColorVariant<C, T>
where <T as ColorFamily<C>>::Data<C>: Clone,
      <T as ColorFamily<C>>::Data<La<C>>: Clone,
      <T as ColorFamily<C>>::Data<Rgb<C>>: Clone,
      <T as ColorFamily<C>>::Data<Rgba<C>>: Clone {
    #[inline]
    fn clone(&self) -> Self {
        match self {
            Self::L(data)    => Self::L(data.clone()),
            Self::La(data)   => Self::La(data.clone()),
            Self::Rgb(data)  => Self::Rgb(data.clone()),
            Self::Rgba(data) => Self::Rgba(data.clone()),
        }
    }
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

impl ColorList {
    #[inline]
    pub fn color_type(&self) -> ColorType {
        match self {
            ChannelVariant::U8(  data) => data.color_type(),
            ChannelVariant::U16 (data) => data.color_type(),
            ChannelVariant::U32 (data) => data.color_type(),
            ChannelVariant::U64 (data) => data.color_type(),
            ChannelVariant::U128(data) => data.color_type(),
            ChannelVariant::F32 (data) => data.color_type(),
            ChannelVariant::F64 (data) => data.color_type(),
        }
    }
}

pub fn read_colors_into<Color, ChannelValue>(mut bytes: &[u8], colors: &mut Vec<Color>)
where Color: crate::color::Color<ChannelValue>,
      ChannelValue: crate::color::ChannelValue
{
    let color_size = Color::CHANNELS as usize * ChannelValue::SIZE as usize;
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
                        colors.push(((byte >> 0) & 1) * 255);
                        colors.push(((byte >> 1) & 1) * 255);
                        colors.push(((byte >> 2) & 1) * 255);
                        colors.push(((byte >> 3) & 1) * 255);
                        colors.push(((byte >> 4) & 1) * 255);
                        colors.push(((byte >> 5) & 1) * 255);
                        colors.push(((byte >> 6) & 1) * 255);
                        colors.push(((byte >> 7) & 1) * 255);
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
                        let v = get_nibble(bytes, offset); let v = (v << 4) | v;
                        colors.push(v);
                    }
                    Ok(ChannelVariant::U8(ColorVariant::L(colors)))
                },
                3 => {
                    let color_count = (bytes.len() * 2) / 3;
                    let mut colors = Vec::with_capacity(color_count);
                    for offset in (0..(color_count * 3)).step_by(3) {
                        let r = get_nibble(bytes, offset + 0); let r = (r << 4) | r;
                        let g = get_nibble(bytes, offset + 1); let g = (g << 4) | g;
                        let b = get_nibble(bytes, offset + 2); let b = (b << 4) | b;
                        colors.push(Rgb([r, g, b]));
                    }
                    Ok(ChannelVariant::U8(ColorVariant::Rgb(colors)))
                },
                4 => {
                    let color_count = (bytes.len() * 2) / 4;
                    let mut colors = Vec::with_capacity(color_count);
                    for offset in (0..(color_count * 4)).step_by(4) {
                        let r = get_nibble(bytes, offset + 0); let r = (r << 4) | r;
                        let g = get_nibble(bytes, offset + 1); let g = (g << 4) | g;
                        let b = get_nibble(bytes, offset + 2); let b = (b << 4) | b;
                        let a = get_nibble(bytes, offset + 3); let a = (a << 4) | a;
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

#[inline]
pub fn write_colors_variant(colors: &ColorList, planes: u8, writer: &mut impl Write) -> Result<(), WriteError> {
    match colors {
        // TODO: correctly handle planes if <= 8
        ChannelVariant::U8  (colors) if planes ==   1 => write_1bit_colors_variant_inner(colors, writer)?,
        ChannelVariant::U8  (colors) if planes ==   4 => write_4bit_colors_variant_inner(colors, writer)?,
        ChannelVariant::U8  (colors) if planes ==   8 => write_colors_variant_inner(colors, writer)?,
        ChannelVariant::U16 (colors) if planes ==  16 => write_colors_variant_inner(colors, writer)?,
        ChannelVariant::U32 (colors) if planes ==  32 => write_colors_variant_inner(colors, writer)?,
        ChannelVariant::U64 (colors) if planes ==  64 => write_colors_variant_inner(colors, writer)?,
        ChannelVariant::U128(colors) if planes == 128 => write_colors_variant_inner(colors, writer)?,
        ChannelVariant::F32 (colors) if planes ==  32 => write_colors_variant_inner(colors, writer)?,
        ChannelVariant::F64 (colors) if planes ==  64 => write_colors_variant_inner(colors, writer)?,
        _ => {
            return Err(WriteError::with_message(
                WriteErrorKind::InvalidParams,
                format!("invalid bit depth for channel value type: {planes} Vs {}", colors.channel_value_type())
            ));
        }
    }

    Ok(())
}

#[inline]
pub fn write_1bit_colors_variant_inner(colors: &ColorVariant<u8, ColorVecDataInner>, writer: &mut impl Write) -> std::io::Result<()> {
    match colors {
        ColorVariant::L   (colors) => write_1bit_colors(colors, writer),
        ColorVariant::La  (colors) => write_1bit_colors(colors, writer),
        ColorVariant::Rgb (colors) => write_1bit_colors(colors, writer),
        ColorVariant::Rgba(colors) => write_1bit_colors(colors, writer),
    }
}

#[inline]
pub fn write_1bit_colors<Color>(colors: &[Color], mut writer: impl Write) -> std::io::Result<()>
where Color: crate::color::Color<u8>
{
    let mut byte = 0u8;
    let mut bit = 0;

    for color in colors {
        for &channel in color.channels() {
            if bit == 8 {
                writer.write_all(std::slice::from_ref(&byte))?;
                byte = 0;
            }
            byte |= channel << bit;
            bit += 1;
        }
    }

    if bit != 0 {
        writer.write_all(std::slice::from_ref(&byte))?;
    }

    Ok(())
}

#[inline]
pub fn write_4bit_colors_variant_inner(colors: &ColorVariant<u8, ColorVecDataInner>, writer: &mut impl Write) -> std::io::Result<()> {
    match colors {
        ColorVariant::L   (colors) => write_4bit_colors(colors, writer),
        ColorVariant::La  (colors) => write_4bit_colors(colors, writer),
        ColorVariant::Rgb (colors) => write_4bit_colors(colors, writer),
        ColorVariant::Rgba(colors) => write_4bit_colors(colors, writer),
    }
}

#[inline]
pub fn write_4bit_colors<Color>(colors: &[Color], mut writer: impl Write) -> std::io::Result<()>
where Color: crate::color::Color<u8>
{
    let mut byte = 0u8;
    let mut bit = 0;

    for color in colors {
        for &channel in color.channels() {
            if bit == 8 {
                writer.write_all(std::slice::from_ref(&byte))?;
                byte = channel << 4;
            } else {
                byte |= channel;
            }
            bit += 4;
        }
    }

    if bit != 0 {
        writer.write_all(std::slice::from_ref(&byte))?;
    }

    Ok(())
}

#[inline]
pub fn write_colors_variant_inner<C: crate::color::ChannelValue>(colors: &ColorVariant<C, ColorVecDataInner>, writer: &mut impl Write) -> std::io::Result<()> {
    match colors {
        ColorVariant::L   (colors) => write_colors(colors, writer),
        ColorVariant::La  (colors) => write_colors(colors, writer),
        ColorVariant::Rgb (colors) => write_colors(colors, writer),
        ColorVariant::Rgba(colors) => write_colors(colors, writer),
    }
}

#[inline]
pub fn write_colors<Color, ChannelValue>(colors: &[Color], mut writer: impl Write) -> std::io::Result<()>
where ChannelValue: crate::color::ChannelValue,
      Color: crate::color::Color<ChannelValue>,
{
    for color in colors {
        color.write_to(&mut writer)?;
    }
    Ok(())
}

pub fn apply_palette<ChannelValue: self::ChannelValue, Color: self::Color<ChannelValue>>(img: &[impl TryInto<usize> + Copy], palette: &[Color]) -> Vec<Color> {
    let mut output = Vec::with_capacity(img.len());

    for index in img {
        if let Ok(index) = TryInto::<usize>::try_into(*index) {
            if let Some(color) = palette.get(index) {
                output.push(color.clone());
            } else {
                output.push(Color::default());
            }
        } else {
            output.push(Color::default());
        }
    }

    output
}

pub fn apply_palette_variant_inner<C: ChannelValue>(img: &[impl TryInto<usize> + Copy], palette: &ColorVariant<C, ColorVecDataInner>) -> ColorVariant<C, ColorVecDataInner> {
    match palette {
        ColorVariant::L   (palette) => ColorVariant::L   (apply_palette(img, &palette[..])),
        ColorVariant::La  (palette) => ColorVariant::La  (apply_palette(img, &palette[..])),
        ColorVariant::Rgb (palette) => ColorVariant::Rgb (apply_palette(img, &palette[..])),
        ColorVariant::Rgba(palette) => ColorVariant::Rgba(apply_palette(img, &palette[..])),
    }
}

pub fn apply_palette_variant(img: &[impl TryInto<usize> + Copy], palette: &ColorList) -> ColorList {
    match palette {
        ChannelVariant::U8  (palette) => ChannelVariant::U8  (apply_palette_variant_inner(img, palette)),
        ChannelVariant::U16 (palette) => ChannelVariant::U16 (apply_palette_variant_inner(img, palette)),
        ChannelVariant::U32 (palette) => ChannelVariant::U32 (apply_palette_variant_inner(img, palette)),
        ChannelVariant::U64 (palette) => ChannelVariant::U64 (apply_palette_variant_inner(img, palette)),
        ChannelVariant::U128(palette) => ChannelVariant::U128(apply_palette_variant_inner(img, palette)),
        ChannelVariant::F32 (palette) => ChannelVariant::F32 (apply_palette_variant_inner(img, palette)),
        ChannelVariant::F64 (palette) => ChannelVariant::F64 (apply_palette_variant_inner(img, palette)),
    }
}
