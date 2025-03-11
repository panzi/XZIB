use crate::{color::{read_colors_variant, ColorList}, error::{ReadError, ReadErrorKind}, format::Format, Head};

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

// TODO: color mapping https://threadlocalmutex.com/?p=60 https://threadlocalmutex.com/?p=48
pub fn read_interleaved_colors(bytes: &[u8], is_float: bool, planes: u8, channels: u8, width: u32, height: u32) -> Result<ColorList, ReadError> {
    if channels != 1 && channels != 3 && channels != 4 {
        return Err(ReadError::with_message(
            ReadErrorKind::BrokenFile, format!("illegal number of channels: {channels}")));
    }

    let mut line_buf: Vec<u8> = Vec::with_capacity(width as usize * channels as usize * planes as usize);
    let plane_len = (width as usize + 7) / 8;

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
    } else if planes <= 8 {
        for y in 0..height {
            for x in 0..width {
                let value = 0u8;
                for plane in 0..planes {
                    
                }
            }
        }
        todo!()
    } else if planes <= 16 {
        todo!()
    } else if planes <= 32 {
        todo!()
    } else if planes <= 64 {
        todo!()
    } else if planes <= 128 {
        todo!()
    } else {
        return Err(ReadError::with_message(
            ReadErrorKind::BrokenFile,
            format!("unsupported color format: {} {planes}", if is_float { "float" } else { "int" })));
    }
}
