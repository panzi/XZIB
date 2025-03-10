use std::{io::{Read, Write}, mem::MaybeUninit};

#[inline]
pub fn read_u8(reader: &mut impl Read) -> Result<u8, std::io::Error> {
    let mut buf = MaybeUninit::<[u8; 1]>::uninit();
    reader.read_exact(unsafe { buf.assume_init_mut() })?;
    Ok(unsafe { buf.assume_init_ref() }[0])
}

#[inline]
pub fn read_u16(reader: &mut impl Read) -> Result<u16, std::io::Error> {
    let mut buf = MaybeUninit::<[u8; 2]>::uninit();
    reader.read_exact(unsafe { buf.assume_init_mut() })?;
    Ok(u16::from_le_bytes(unsafe { buf.assume_init() }))
}

#[inline]
pub fn read_u32(reader: &mut impl Read) -> Result<u32, std::io::Error> {
    let mut buf = MaybeUninit::<[u8; 4]>::uninit();
    reader.read_exact(unsafe { buf.assume_init_mut() })?;
    Ok(u32::from_le_bytes(unsafe { buf.assume_init() }))
}

#[inline]
pub fn read_u64(reader: &mut impl Read) -> Result<u64, std::io::Error> {
    let mut buf = MaybeUninit::<[u8; 8]>::uninit();
    reader.read_exact(unsafe { buf.assume_init_mut() })?;
    Ok(u64::from_le_bytes(unsafe { buf.assume_init() }))
}

#[inline]
pub fn read_u128(reader: &mut impl Read) -> Result<u128, std::io::Error> {
    let mut buf = MaybeUninit::<[u8; 16]>::uninit();
    reader.read_exact(unsafe { buf.assume_init_mut() })?;
    Ok(u128::from_le_bytes(unsafe { buf.assume_init() }))
}

#[inline]
pub fn read_f32(reader: &mut impl Read) -> Result<f32, std::io::Error> {
    let mut buf = MaybeUninit::<[u8; 4]>::uninit();
    reader.read_exact(unsafe { buf.assume_init_mut() })?;
    Ok(f32::from_le_bytes(unsafe { buf.assume_init() }))
}

#[inline]
pub fn read_f64(reader: &mut impl Read) -> Result<f64, std::io::Error> {
    let mut buf = MaybeUninit::<[u8; 8]>::uninit();
    reader.read_exact(unsafe { buf.assume_init_mut() })?;
    Ok(f64::from_le_bytes(unsafe { buf.assume_init() }))
}

#[inline]
pub fn read_fourcc(reader: &mut impl Read) -> Result<[u8; 4], std::io::Error> {
    let mut buf = MaybeUninit::<[u8; 4]>::uninit();
    reader.read_exact(unsafe { buf.assume_init_mut() })?;
    Ok(unsafe { buf.assume_init() })
}

#[inline]
pub fn write_u8(writer: &mut impl Write, value: u8) -> Result<(), std::io::Error> {
    writer.write_all(&[value])
}

#[inline]
pub fn write_u16(writer: &mut impl Write, value: u16) -> Result<(), std::io::Error> {
    writer.write_all(&value.to_le_bytes())
}

#[inline]
pub fn write_u32(writer: &mut impl Write, value: u32) -> Result<(), std::io::Error> {
    writer.write_all(&value.to_le_bytes())
}

#[inline]
pub fn write_u64(writer: &mut impl Write, value: u64) -> Result<(), std::io::Error> {
    writer.write_all(&value.to_le_bytes())
}

#[inline]
pub fn write_u128(writer: &mut impl Write, value: u128) -> Result<(), std::io::Error> {
    writer.write_all(&value.to_le_bytes())
}

#[inline]
pub fn write_f32(writer: &mut impl Write, value: f32) -> Result<(), std::io::Error> {
    writer.write_all(&value.to_le_bytes())
}

#[inline]
pub fn write_f64(writer: &mut impl Write, value: f64) -> Result<(), std::io::Error> {
    writer.write_all(&value.to_le_bytes())
}

#[inline]
pub fn write_fourcc(writer: &mut impl Write, value: [u8; 4]) -> Result<(), std::io::Error> {
    writer.write_all(&value)
}
