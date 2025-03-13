use crate::color::{ChannelValue, ChannelVariant, ColorList, ColorVariant, ColorVecDataInner};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NumberType {
    Integer,
    Float,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ChannelValueType {
    U8, U16, U32, U64, U128,
    F32, F64,
}

impl ChannelValueType {
    #[inline]
    pub fn from_planes(number_type: NumberType, planes: u8) -> Option<Self> {
        match number_type {
            NumberType::Integer => {
                if planes <= 8 { Some(Self::U8) }
                else if planes <= 16 { Some(Self::U16) }
                else if planes <= 32 { Some(Self::U32) }
                else if planes <= 64 { Some(Self::U64) }
                else if planes <= 128 { Some(Self::U128) }
                else { None }
            }
            NumberType::Float => {
                if planes <= 32 { Some(Self::F32) }
                else if planes <= 64 { Some(Self::F64) }
                else { None }
            }
        }
    }

    #[inline]
    pub fn planes(self) -> u8 {
        match self {
            Self::U8 => 8,
            Self::U16 => 16,
            Self::U32 | Self::F32 => 32,
            Self::U64 | Self::F64 => 64,
            Self::U128 => 128,
        }
    }

    #[inline]
    pub fn number_type(self) -> NumberType {
        match self {
            Self::U8 |
            Self::U16 |
            Self::U32 |
            Self::U64 |
            Self::U128 => NumberType::Integer,

            Self::F32 |
            Self::F64 => NumberType::Float
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ColorType {
    L, Rgb, Rgba
}

impl ColorType {
    #[inline]
    pub fn from_channels(channels: u8) -> Option<Self> {
        match channels {
            1 => Some(Self::L),
            3 => Some(Self::Rgb),
            4 => Some(Self::Rgba),
            _ => None,
        }
    }

    #[inline]
    pub fn channels(self) -> u8 {
        match self {
            Self::L    => 1,
            Self::Rgb  => 3,
            Self::Rgba => 4,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Format(pub ChannelValueType, pub ColorType);

impl Format {
    #[inline]
    pub fn from_components(number_type: NumberType, planes: u8, channels: u8) -> Option<Self> {
        let Some(channel_value_type) = ChannelValueType::from_planes(number_type, planes) else {
            return None;
        };

        let Some(color_type) = ColorType::from_channels(channels) else {
            return None;
        };

        Some(Format(channel_value_type, color_type))
    }

    #[inline]
    fn make_color_list_inner<C: ChannelValue>(color_type: ColorType) -> ColorVariant<C, ColorVecDataInner> {
        match color_type {
            ColorType::L    => ColorVariant::L(Vec::new()),
            ColorType::Rgb  => ColorVariant::Rgb(Vec::new()),
            ColorType::Rgba => ColorVariant::Rgba(Vec::new()),
        }
    }

    #[inline]
    pub fn make_color_list(&self) -> ColorList {
        let Format(channel_value_type, color_type) = *self;
        match channel_value_type {
            ChannelValueType::U8   => ChannelVariant::U8(Self::make_color_list_inner(color_type)),
            ChannelValueType::U16  => ChannelVariant::U16(Self::make_color_list_inner(color_type)),
            ChannelValueType::U32  => ChannelVariant::U32(Self::make_color_list_inner(color_type)),
            ChannelValueType::U64  => ChannelVariant::U64(Self::make_color_list_inner(color_type)),
            ChannelValueType::U128 => ChannelVariant::U128(Self::make_color_list_inner(color_type)),
            ChannelValueType::F32  => ChannelVariant::F32(Self::make_color_list_inner(color_type)),
            ChannelValueType::F64  => ChannelVariant::F64(Self::make_color_list_inner(color_type)),
        }
    }
}
