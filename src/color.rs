use std::ops::{Add, AddAssign, Mul, MulAssign, Sub, SubAssign};


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
    const MAX_VALUE: Self;
}

impl ChannelValue for u8 {
    const ZERO: Self = 0;
    const MAX_VALUE: Self = 0xFF;
}

impl ChannelValue for u16 {
    const ZERO: Self = 0;
    const MAX_VALUE: Self = 0xFFFF;
}

impl ChannelValue for u32 {
    const ZERO: Self = 0;
    const MAX_VALUE: Self = 0xFFFF_FFFF;
}

impl ChannelValue for u64 {
    const ZERO: Self = 0;
    const MAX_VALUE: Self = 0xFFFF_FFFF_FFFF_FFFF;
}

impl ChannelValue for u128 {
    const ZERO: Self = 0;
    const MAX_VALUE: Self = 0xFFFF_FFFF_FFFF_FFFF_FFFF_FFFF_FFFF_FFFF;
}

// TODO: how to #[cfg()] check this?
//impl ChannelValue for f16 {}
//impl ChannelValue for f128 {}

impl ChannelValue for f32 {
    const ZERO: Self = 0.0;
    const MAX_VALUE: Self = 1.0;
}

impl ChannelValue for f64 {
    const ZERO: Self = 0.0;
    const MAX_VALUE: Self = 1.0;
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
#[repr(transparent)]
pub struct Rgb<C: ChannelValue>(pub [C; 3]);

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
#[repr(transparent)]
pub struct Rgba<C: ChannelValue>(pub [C; 4]);

pub trait Color<C: ChannelValue>: std::fmt::Debug {
    fn to_rgb(&self) -> Rgb<C>;
    fn to_rgba(&self) -> Rgba<C>;
}

impl<C: ChannelValue> Color<C> for Rgb<C> {
    #[inline]
    fn to_rgb(&self) -> Rgb<C> {
        *self
    }

    #[inline]
    fn to_rgba(&self) -> Rgba<C> {
        let Rgb([r, g, b]) = *self;
        Rgba([r, g, b, C::MAX_VALUE])
    }
}

impl<C: ChannelValue> Color<C> for Rgba<C> {
    #[inline]
    fn to_rgb(&self) -> Rgb<C> {
        let Rgba([r, g, b, _]) = *self;
        Rgb([r, g, b])
    }

    #[inline]
    fn to_rgba(&self) -> Rgba<C> {
        *self
    }
}

impl<C: ChannelValue> Color<C> for C {
    #[inline]
    fn to_rgb(&self) -> Rgb<C> {
        Rgb([*self, *self, *self])
    }

    #[inline]
    fn to_rgba(&self) -> Rgba<C> {
        Rgba([*self, *self, *self, C::MAX_VALUE])
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
