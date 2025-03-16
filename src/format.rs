use crate::{color::{ChannelValue, ChannelVariant, ColorList, ColorVariant, ColorVecDataInner}, error::InvalidParams};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NumberType {
    Integer,
    Float,
}

impl NumberType {
    #[inline]
    pub fn is_float(self) -> bool {
        matches!(self, NumberType::Float)
    }

    #[inline]
    pub fn is_integer(self) -> bool {
        matches!(self, NumberType::Integer)
    }
}

impl std::fmt::Display for NumberType {
    #[inline]
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Integer => "int".fmt(f),
            Self::Float   => "float".fmt(f)
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ChannelValueType {
    U8, U16, U32, U64, U128,
    F32, F64,
}

impl ChannelValueType {
    #[inline]
    pub fn from_planes(number_type: NumberType, planes: u8) -> Result<Self, InvalidParams> {
        match number_type {
            NumberType::Integer => {
                if planes <= 8 { Ok(Self::U8) }
                else if planes <= 16 { Ok(Self::U16) }
                else if planes <= 32 { Ok(Self::U32) }
                else if planes <= 64 { Ok(Self::U64) }
                else if planes <= 128 { Ok(Self::U128) }
                else {
                    Err(InvalidParams::with_message(
                        format!("invalid number of planes for number type {}: {}", number_type, planes)
                    ))
                }
            }
            NumberType::Float => {
                if planes <= 32 { Ok(Self::F32) }
                else if planes <= 64 { Ok(Self::F64) }
                else {
                    Err(InvalidParams::with_message(
                        format!("invalid number of planes for number type {}: {}", number_type, planes)
                    ))
                }
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

impl std::fmt::Display for ChannelValueType {
    #[inline]
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} {}", self.number_type(), self.planes())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ColorType {
    L, La, Rgb, Rgba
}

impl ColorType {
    #[inline]
    pub fn from_channels(channels: u8) -> Result<Self, InvalidParams> {
        match channels {
            1 => Ok(Self::L),
            3 => Ok(Self::Rgb),
            4 => Ok(Self::Rgba),
            _ => Err(InvalidParams::with_message(format!("invalid number of channels: {channels}"))),
        }
    }

    #[inline]
    pub fn channels(self) -> u8 {
        match self {
            Self::L    => 1,
            Self::La   => 2,
            Self::Rgb  => 3,
            Self::Rgba => 4,
        }
    }
}

impl std::fmt::Display for ColorType {
    #[inline]
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::L    => "L".fmt(f),
            Self::La   => "LA".fmt(f),
            Self::Rgb  => "RGB".fmt(f),
            Self::Rgba => "RGBA".fmt(f),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Format(pub ChannelValueType, pub ColorType);

impl Format {
    #[inline]
    pub fn from_components(number_type: NumberType, planes: u8, channels: u8) -> Result<Self, InvalidParams> {
        let channel_value_type = ChannelValueType::from_planes(number_type, planes)?;
        let color_type = ColorType::from_channels(channels)?;

        Ok(Format(channel_value_type, color_type))
    }

    #[inline]
    fn make_color_list_inner<C: ChannelValue>(color_type: ColorType) -> ColorVariant<C, ColorVecDataInner> {
        match color_type {
            ColorType::L    => ColorVariant::L   (Vec::new()),
            ColorType::La   => ColorVariant::La  (Vec::new()),
            ColorType::Rgb  => ColorVariant::Rgb (Vec::new()),
            ColorType::Rgba => ColorVariant::Rgba(Vec::new()),
        }
    }

    #[inline]
    pub fn make_color_list(&self) -> ColorList {
        let Format(channel_value_type, color_type) = *self;
        match channel_value_type {
            ChannelValueType::U8   => ChannelVariant::U8  (Self::make_color_list_inner(color_type)),
            ChannelValueType::U16  => ChannelVariant::U16 (Self::make_color_list_inner(color_type)),
            ChannelValueType::U32  => ChannelVariant::U32 (Self::make_color_list_inner(color_type)),
            ChannelValueType::U64  => ChannelVariant::U64 (Self::make_color_list_inner(color_type)),
            ChannelValueType::U128 => ChannelVariant::U128(Self::make_color_list_inner(color_type)),
            ChannelValueType::F32  => ChannelVariant::F32 (Self::make_color_list_inner(color_type)),
            ChannelValueType::F64  => ChannelVariant::F64 (Self::make_color_list_inner(color_type)),
        }
    }
}

impl std::fmt::Display for Format {
    #[inline]
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let Format(channel_value_type, color_type) = self;
        write!(f, "{color_type} {channel_value_type}")
    }
}
