use std::io::Write;

use crate::{color::{read_colors_variant, ChannelVariant, ColorList, ColorVariant, ColorVecDataInner}, error::{ReadError, ReadErrorKind, WriteError, WriteErrorKind}, format::ChannelValueType, Head};

use super::ChunkWrite;

#[derive(Debug)]
pub struct Indx {
    colors: ColorList
}

impl Indx {
    pub const FOURCC: [u8; 4] = *b"INDX";

    #[inline]
    pub fn colors(&self) -> &ColorList {
        &self.colors
    }

    #[inline]
    pub fn colors_mut(&mut self) -> &mut ColorList {
        &mut self.colors
    }

    pub fn read(data: &[u8], head: &Head) -> Result<Self, ReadError> {
        let index_planes = head.index_planes();
        if index_planes == 0 {
            return Err(ReadError::with_message(
                ReadErrorKind::BrokenFile,
                format!("indx chunk found, but index_planes == 0")
            ));
        }

        let colors = read_colors_variant(data, head.is_float(), head.index_planes(), head.channels())?;

        Ok(Self { colors })
    }

    pub fn write(&self, head: &Head, writer: &mut impl Write) -> Result<(), WriteError> {
        let Some(channel_value_type) = ChannelValueType::from_planes(head.number_type(), head.index_planes()) else {
            return Err(WriteError::with_message(
                WriteErrorKind::InvalidParams,
                format!("unsupported parameters for index colors: {} {}", head.number_type(), head.index_planes())));
        };

        if self.colors.channel_value_type() != channel_value_type {
            return Err(WriteError::with_message(
                WriteErrorKind::InvalidParams,
                format!("actual index channel value type doesn't match type defined in header: {} != {}", self.colors.channel_value_type(), channel_value_type)));
        }

        write_colors_variant(&self.colors, head.index_planes(), writer)?;

        Ok(())
    }
}

#[inline]
pub fn write_colors_variant(colors: &ColorList, planes: u8, writer: &mut impl Write) -> Result<(), WriteError> {
    match colors {
        // TODO: correctly handle planes if <= 8
        ChannelVariant::U8  (colors) if planes ==   1 => todo!(),
        ChannelVariant::U8  (colors) if planes ==   4 => todo!(),
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
pub fn write_colors_variant_inner<C: crate::color::ChannelValue>(colors: &ColorVariant<C, ColorVecDataInner>, writer: &mut impl Write) -> std::io::Result<()> {
    match colors {
        ColorVariant::L   (colors) => write_colors(colors, writer),
        ColorVariant::Rgb (colors) => write_colors(colors, writer),
        ColorVariant::Rgba(colors) => write_colors(colors, writer),
    }
}

#[inline]
pub fn write_colors<Color, ChannelValue>(colors: &[Color], mut writer: &mut impl Write) -> std::io::Result<()>
where ChannelValue: crate::color::ChannelValue,
      Color: crate::color::Color<ChannelValue>,
{
    for color in colors {
        color.write_to(&mut writer)?;
    }
    Ok(())
}

impl ChunkWrite for Indx {
    const FOURCC: [u8; 4] = Self::FOURCC;

    #[inline]
    fn write(&self, head: &Head, writer: &mut impl Write) -> Result<(), WriteError> {
        self.write(head, writer)
    }
}
