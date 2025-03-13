use crate::{color::{read_colors_variant, ChannelVariant, ColorList, ColorVariant, ColorVecDataInner, IntChannelValue, Rgb, Rgba}, error::{ReadError, ReadErrorKind}, format::Format, Head};

#[derive(Debug)]
pub struct Body {
    data: ColorList
}

impl Body {
    #[inline]
    pub fn new(format: Format) -> Self {
        Self { data: format.make_color_list() }
    }

    #[inline]
    pub fn data(&self) -> &ColorList {
        &self.data
    }

    #[inline]
    pub fn data_mut(&mut self) -> &mut ColorList {
        &mut self.data
    }

    pub fn read(bytes: &[u8], head: &Head) -> Result<Self, ReadError> {
        if !head.is_interleaved() {
            return Ok(Self {
                data: read_colors_variant(bytes, head.is_float(), head.planes(), head.channels())?
            });
        }

        let data = read_interleaved_colors(bytes, head.is_float(), head.planes(), head.channels(), head.width(), head.height())?;

        Ok(Self {
            data
        })
    }
}


// color mapping https://threadlocalmutex.com/?p=60 https://threadlocalmutex.com/?p=48
// XXX: however, I just shift it instead
pub const LOOKUP_8: [fn(u8) -> u8; 8] = [
    /*  1 */ |x: u8| x * 255,
    /*  2 */ |x: u8| x * 85,
    /*  3 */ |x: u8| (x * 146 + 1) >> 2,
    /*  4 */ |x: u8| x * 17,
    /*  5 */ |x: u8| ((x as u32 * 527 + 23) >> 6) as u8,
    /*  6 */ |x: u8| ((x as u32 * 259 + 33) >> 6) as u8,
    /*  7 */ |x: u8| ((x as u32 * 257 + 64) >> 7) as u8,
    /*  8 */ |x: u8| x,
];

pub const LOOKUP_16: [fn(u16) -> u16; 16] = [
    /*  1 */ |x: u16| x * 0xFFFF,
    /*  2 */ |x: u16| x * 21845,
    /*  3 */ |x: u16| ((x as u64 * 74897 + 4) >> 3) as u16,
    /*  4 */ |x: u16| x * 4369,
    /*  5 */ |x: u16| ((x as u64 * 67649 + 16) >> 5) as u16,
    /*  6 */ |x: u16| ((x as u64 * 266301 + 128) >> 8) as u16,
    /*  7 */ |x: u16| ((x as u64 * 1056817 + 992) >> 11) as u16,
    /*  8 */ |x: u16| x * 257,
    /*  9 */ |x: u16| ((x as u64 * 262653 + 1028) >> 11) as u16,
    /* 10 */ |x: u16| ((x as u64 * 1049585 + 8208) >> 14) as u16,

    // XXX: I dunno
    /* 11 */ |x: u16| x << 5,
    /* 12 */ |x: u16| x << 4,
    /* 13 */ |x: u16| x << 3,
    /* 14 */ |x: u16| x << 2,
    /* 15 */ |x: u16| x << 1,

    /* 16 */ |x: u16| x,
];

pub fn read_interleaved_colors(bytes: &[u8], is_float: bool, planes: u8, channels: u8, width: u32, height: u32) -> Result<ColorList, ReadError> {
    if channels != 1 && channels != 3 && channels != 4 {
        return Err(ReadError::with_message(
            ReadErrorKind::BrokenFile, format!("illegal number of channels: {channels}")));
    }

    if is_float {
        match planes {
            32 => {
                todo!()
            }
            64 => {
                todo!()
            }
            _ => {
                return Err(ReadError::with_message(
                    ReadErrorKind::BrokenFile,
                    format!("unsupported color format: {} {planes}", if is_float { "float" } else { "int" })));
            }
        }
    }

    if planes == 0 {
        return Err(ReadError::with_message(
            ReadErrorKind::BrokenFile,
            format!("unsupported color format: {} {planes}", if is_float { "float" } else { "int" })));
    } else if planes == 1 {
        // special case for 1-bit black and white images
        // here the bits aren't just shifted to the left, but its mapped to 0 or 255
        Ok(ChannelVariant::U8(read_1bit_colors(bytes, channels, width, height)?))
    } else if planes <= 8 {
        Ok(ChannelVariant::U8(read_interleaved_int_colors(bytes, planes, channels, width, height)?))
    } else if planes <= 16 {
        Ok(ChannelVariant::U16(read_interleaved_int_colors(bytes, planes, channels, width, height)?))
    } else if planes <= 32 {
        Ok(ChannelVariant::U32(read_interleaved_int_colors(bytes, planes, channels, width, height)?))
    } else if planes <= 64 {
        Ok(ChannelVariant::U64(read_interleaved_int_colors(bytes, planes, channels, width, height)?))
    } else if planes <= 128 {
        Ok(ChannelVariant::U128(read_interleaved_int_colors(bytes, planes, channels, width, height)?))
    } else {
        return Err(ReadError::with_message(
            ReadErrorKind::BrokenFile,
            format!("unsupported color format: {} {planes}", if is_float { "float" } else { "int" })));
    }
}

pub fn read_interleaved_int_colors<C: IntChannelValue>(bytes: &[u8], planes: u8, channels: u8, width: u32, height: u32) -> Result<ColorVariant<C, ColorVecDataInner>, ReadError> {
    let plane_len = (width as usize + 7) / 8;
    let channel_len = plane_len * planes as usize;
    let row_len = channel_len * channels as usize;
    let shift = C::SIZE * 8 - planes as usize;
    match channels {
        1 => {
            let mut data = Vec::with_capacity(width as usize * height as usize);
            for y in 0..height as usize {
                let y_offset = y * row_len;
                let bytes = &bytes[y_offset..];

                for x in 0..width as usize {
                    let value: C = read_interleaved_int_color(bytes, planes, plane_len, x);
                    data.push(value << shift);
                }
            }
            Ok(ColorVariant::L(data))
        }
        3 => {
            let mut data = Vec::with_capacity(width as usize * height as usize);
            for y in 0..height as usize {
                let y_offset = y * row_len;
                let reds   = &bytes[y_offset..];
                let greens = &reds[channel_len..];
                let blues  = &greens[channel_len..];

                for x in 0..width as usize {
                    let r = read_interleaved_int_color::<C>(reds,   planes, plane_len, x) << shift;
                    let g = read_interleaved_int_color::<C>(greens, planes, plane_len, x) << shift;
                    let b = read_interleaved_int_color::<C>(blues,  planes, plane_len, x) << shift;

                    data.push(Rgb([r, g, b]));
                }
            }
            Ok(ColorVariant::Rgb(data))
        }
        4 => {
            let mut data = Vec::with_capacity(width as usize * height as usize);
            for y in 0..height as usize {
                let y_offset = y * row_len;
                let reds   = &bytes[y_offset..];
                let greens = &reds[channel_len..];
                let blues  = &greens[channel_len..];
                let alphas = &blues[channel_len..];

                for x in 0..width as usize {
                    let r = read_interleaved_int_color::<C>(reds,   planes, plane_len, x) << shift;
                    let g = read_interleaved_int_color::<C>(greens, planes, plane_len, x) << shift;
                    let b = read_interleaved_int_color::<C>(blues,  planes, plane_len, x) << shift;
                    let a = read_interleaved_int_color::<C>(alphas, planes, plane_len, x) << shift;

                    data.push(Rgba([r, g, b, a]));
                }
            }
            Ok(ColorVariant::Rgba(data))
        }
        _ => Err(ReadError::with_message(
            ReadErrorKind::BrokenFile,
            format!("illegal number of channels: {channels}")))
    }
}

#[inline]
pub fn read_interleaved_int_color<C: IntChannelValue>(bytes: &[u8], planes: u8, plane_len: usize, x: usize) -> C {
    let mut value = C::ZERO;
    let byte_offset = x / 8;
    let bit_offset = x % 8;
    let byte_offset = byte_offset;
    for plane in 0..planes as usize {
        let plane_index = plane_len * plane;
        value |= ((bytes[plane_index + byte_offset] >> bit_offset) << plane).into();
    }
    value
}

pub fn read_1bit_colors(bytes: &[u8], channels: u8, width: u32, height: u32) -> Result<ColorVariant<u8, ColorVecDataInner>, ReadError> {
    let plane_len = (width as usize + 7) / 8;
    let channel_len = plane_len;
    let row_len = channel_len * channels as usize;
    match channels {
        1 => {
            let mut data = Vec::with_capacity(width as usize * height as usize);
            for y in 0..height as usize {
                let y_offset = y * row_len;
                let bytes = &bytes[y_offset..];

                for x in 0..width as usize {
                    data.push(read_1bit_color(bytes, x) * 255);
                }
            }
            Ok(ColorVariant::L(data))
        }
        3 => {
            let mut data = Vec::with_capacity(width as usize * height as usize);
            for y in 0..height as usize {
                let y_offset = y * row_len;
                let reds   = &bytes[y_offset..];
                let greens = &reds[channel_len..];
                let blues  = &greens[channel_len..];

                for x in 0..width as usize {
                    let r = read_1bit_color(reds,   x) * 255;
                    let g = read_1bit_color(greens, x) * 255;
                    let b = read_1bit_color(blues,  x) * 255;

                    data.push(Rgb([r, g, b]));
                }
            }
            Ok(ColorVariant::Rgb(data))
        }
        4 => {
            let mut data = Vec::with_capacity(width as usize * height as usize);
            for y in 0..height as usize {
                let y_offset = y * row_len;
                let reds   = &bytes[y_offset..];
                let greens = &reds[channel_len..];
                let blues  = &greens[channel_len..];
                let alphas = &blues[channel_len..];

                for x in 0..width as usize {
                    let r = read_1bit_color(reds,   x) * 255;
                    let g = read_1bit_color(greens, x) * 255;
                    let b = read_1bit_color(blues,  x) * 255;
                    let a = read_1bit_color(alphas, x) * 255;

                    data.push(Rgba([r, g, b, a]));
                }
            }
            Ok(ColorVariant::Rgba(data))
        }
        _ => Err(ReadError::with_message(
            ReadErrorKind::BrokenFile,
            format!("illegal number of channels: {channels}")))
    }
}

#[inline]
pub fn read_1bit_color(bytes: &[u8], x: usize) -> u8 {
    let byte_offset = x / 8;
    let bit_offset = x % 8;
    let byte_offset = byte_offset;
    let value = bytes[byte_offset] >> bit_offset;
    value
}
