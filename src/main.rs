use std::{fs::File, io::{BufReader, BufWriter}, path::PathBuf};

use clap::{Parser, Subcommand};

use xzib::{chunks::Body, color::{ChannelValue, ChannelVariant, ColorList, ColorVariant, La, Rgb, Rgba}, make_error, XZIB};

#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    Encode {
        #[clap(short, long, default_value_t = flate2::Compression::default().level())]
        compression: u32,

        #[clap(short, long, default_value = None)]
        planes: Option<u8>,

        #[clap(short, long, default_value_t = true, num_args = 1)]
        interleaved: bool,

        #[clap()]
        input: PathBuf,

        #[clap()]
        output: PathBuf,
    },

    Decode {
        #[clap()]
        input: PathBuf,

        #[clap()]
        output: PathBuf,
    },

    Info {
        files: Vec<PathBuf>,
    },
}

make_error! {
    CliError;
    struct CliErrorInner {}
    enum CliErrorKind {
        IO,
        Image,
        InvalidParams,
        WriteError,
        ReadError,
    }
    impl IO: std::io::Error;
    impl Image: image::ImageError;
    impl InvalidParams: xzib::error::InvalidParams;
    impl WriteError: xzib::error::WriteError;
    impl ReadError: xzib::error::ReadError;
}

pub fn main() -> Result<(), CliError> {
    let args = Cli::parse();

    match args.command {
        Command::Encode { compression, planes, interleaved, input, output } => {
            let img = image::ImageReader::open(input)?.decode()?;
            let width = img.width();
            let height = img.height();

            let data: ColorList = match img.color() {
                image::ColorType::L8 => {
                    ChannelVariant::U8(ColorVariant::L(img.into_luma8().into_vec()))
                },
                image::ColorType::L16 => {
                    ChannelVariant::U16(ColorVariant::L(img.into_luma16().into_vec()))
                },
                image::ColorType::Rgb8 => {
                    ChannelVariant::U8(ColorVariant::Rgb(
                        img.into_rgb8().pixels().into_iter().map(
                            |&image::Rgb(color)| Rgb(color)
                        ).collect()))
                },
                image::ColorType::Rgb16 => {
                    ChannelVariant::U16(ColorVariant::Rgb(
                        img.into_rgb16().pixels().into_iter().map(
                            |&image::Rgb(color)| Rgb(color)
                        ).collect()))
                },
                image::ColorType::Rgba8 => {
                    ChannelVariant::U8(ColorVariant::Rgba(
                        img.into_rgba8().pixels().into_iter().map(
                            |&image::Rgba(color)| Rgba(color)
                        ).collect()))
                },
                image::ColorType::Rgba16 => {
                    ChannelVariant::U16(ColorVariant::Rgba(
                        img.into_rgba16().pixels().into_iter().map(
                            |&image::Rgba(color)| Rgba(color)
                        ).collect()))
                },
                image::ColorType::Rgb32F => {
                    ChannelVariant::F32(ColorVariant::Rgba(
                        img.into_rgba32f().pixels().into_iter().map(
                            |&image::Rgba(color)| Rgba(color)
                        ).collect()))
                },
                image::ColorType::Rgba32F => {
                    ChannelVariant::F32(ColorVariant::Rgba(
                        img.into_rgba32f().pixels().into_iter().map(
                            |&image::Rgba(color)| Rgba(color)
                        ).collect()))
                },
                image::ColorType::La8 => {
                    ChannelVariant::U8(ColorVariant::La(
                        img.into_luma_alpha8().pixels().into_iter().map(
                            |&image::LumaA(color)| La(color)
                        ).collect()))
                },
                image::ColorType::La16 => {
                    ChannelVariant::U16(ColorVariant::La(
                        img.into_luma_alpha16().pixels().into_iter().map(
                            |&image::LumaA(color)| La(color)
                        ).collect()))
                },
                _ => {
                    let bits_per_channel = img.color().bits_per_pixel() / img.color().channel_count() as u16;
                    if img.color().has_alpha() {
                        if bits_per_channel <= 8 {
                            ChannelVariant::U8(ColorVariant::Rgba(
                                img.into_rgba8().pixels().into_iter().map(
                                    |&image::Rgba(color)| Rgba(color)
                                ).collect()))
                        } else if bits_per_channel <= 16 {
                            ChannelVariant::U16(ColorVariant::Rgba(
                                img.into_rgba16().pixels().into_iter().map(
                                    |&image::Rgba(color)| Rgba(color)
                                ).collect()))
                        } else {
                            ChannelVariant::F32(ColorVariant::Rgba(
                                img.into_rgba32f().pixels().into_iter().map(
                                    |&image::Rgba(color)| Rgba(color)
                                ).collect()))
                        }
                    } else {
                        if bits_per_channel <= 8 {
                            ChannelVariant::U8(ColorVariant::Rgb(
                                img.into_rgb8().pixels().into_iter().map(
                                    |&image::Rgb(color)| Rgb(color)
                                ).collect()))
                        } else if bits_per_channel <= 16 {
                            ChannelVariant::U16(ColorVariant::Rgb(
                                img.into_rgb16().pixels().into_iter().map(
                                    |&image::Rgb(color)| Rgb(color)
                                ).collect()))
                        } else {
                            ChannelVariant::F32(ColorVariant::Rgb(
                                img.into_rgb32f().pixels().into_iter().map(
                                    |&image::Rgb(color)| Rgb(color)
                                ).collect()))
                        }
                    }
                }
            };

            let channel_value_type = data.channel_value_type();
            let color_type = data.color_type();

            let mut xzib = XZIB::new(xzib::Head::new(
                channel_value_type.number_type(),
                interleaved,
                color_type,
                planes.unwrap_or(channel_value_type.planes()),
                0, // TODO: index support
                width,
                height)?);

            *xzib.body_mut() = Some(Body::with_data(data));

            print_info(&xzib);

            xzib.write(
                &mut BufWriter::new(File::create(output)?),
                compression)?;
        }
        Command::Decode { input, output } => {
            let xzib = XZIB::read(&mut BufReader::new(File::open(input)?))?;
            let width = xzib.head().width();
            let height = xzib.head().height();

            let Some(input_img) = xzib.into_image_data() else {
                return Err(CliError::with_message(
                    CliErrorKind::ReadError,
                    "file has no BODY chunk"));
            };

            let img: Option<image::DynamicImage> = match input_img {
                ChannelVariant::U8(data) => {
                    match data {
                        ColorVariant::L(data) => {
                            image::ImageBuffer::from_raw(
                                width, height, data
                            ).map(|img| image::DynamicImage::ImageLuma8(img))
                        }
                        ColorVariant::La(data) => {
                            image::ImageBuffer::from_raw(
                                width, height, data.into_iter().flat_map(|La(color)| color).collect()
                            ).map(|img| image::DynamicImage::ImageLumaA8(img))
                        }
                        ColorVariant::Rgb(data) => {
                            image::ImageBuffer::from_raw(
                                width, height, data.into_iter().flat_map(|Rgb(color)| color).collect()
                            ).map(|img| image::DynamicImage::ImageRgb8(img))
                        }
                        ColorVariant::Rgba(data) => {
                            image::ImageBuffer::from_raw(
                                width, height, data.into_iter().flat_map(|Rgba(color)| color).collect()
                            ).map(|img| image::DynamicImage::ImageRgba8(img))
                        }
                    }
                }
                ChannelVariant::U16(data) => {
                    match data {
                        ColorVariant::L(data) => {
                            image::ImageBuffer::from_raw(
                                width, height, data
                            ).map(|img| image::DynamicImage::ImageLuma16(img))
                        }
                        ColorVariant::La(data) => {
                            image::ImageBuffer::from_raw(
                                width, height, data.into_iter().flat_map(|La(color)| color).collect()
                            ).map(|img| image::DynamicImage::ImageLumaA16(img))
                        }
                        ColorVariant::Rgb(data) => {
                            image::ImageBuffer::from_raw(
                                width, height, data.into_iter().flat_map(|Rgb(color)| color).collect()
                            ).map(|img| image::DynamicImage::ImageRgb16(img))
                        }
                        ColorVariant::Rgba(data) => {
                            image::ImageBuffer::from_raw(
                                width, height, data.into_iter().flat_map(|Rgba(color)| color).collect()
                            ).map(|img| image::DynamicImage::ImageRgba16(img))
                        }
                    }
                }
                ChannelVariant::U32(data) => {
                    match data {
                        ColorVariant::L(data) => {
                            image::ImageBuffer::from_raw(
                                width, height, data.into_iter().flat_map(|l| { let l = l.as_f32(); [l, l, l] }).collect()
                            ).map(|img| image::DynamicImage::ImageRgb32F(img))
                        }
                        ColorVariant::La(data) => {
                            image::ImageBuffer::from_raw(
                                width, height, data.into_iter().flat_map(|La([l, a])| { let l = l.as_f32(); [l, l, l, a.as_f32()] }).collect()
                            ).map(|img| image::DynamicImage::ImageRgba32F(img))
                        }
                        ColorVariant::Rgb(data) => {
                            image::ImageBuffer::from_raw(
                                width, height, data.into_iter().flat_map(|Rgb(color)| color.map(ChannelValue::as_f32)).collect()
                            ).map(|img| image::DynamicImage::ImageRgb32F(img))
                        }
                        ColorVariant::Rgba(data) => {
                            image::ImageBuffer::from_raw(
                                width, height, data.into_iter().flat_map(|Rgba(color)| color.map(ChannelValue::as_f32)).collect()
                            ).map(|img| image::DynamicImage::ImageRgba32F(img))
                        }
                    }
                }
                ChannelVariant::U64(data) => {
                    match data {
                        ColorVariant::L(data) => {
                            image::ImageBuffer::from_raw(
                                width, height, data.into_iter().flat_map(|l| { let l = l.as_f32(); [l, l, l] }).collect()
                            ).map(|img| image::DynamicImage::ImageRgb32F(img))
                        }
                        ColorVariant::La(data) => {
                            image::ImageBuffer::from_raw(
                                width, height, data.into_iter().flat_map(|La([l, a])| { let l = l.as_f32(); [l, l, l, a.as_f32()] }).collect()
                            ).map(|img| image::DynamicImage::ImageRgba32F(img))
                        }
                        ColorVariant::Rgb(data) => {
                            image::ImageBuffer::from_raw(
                                width, height, data.into_iter().flat_map(|Rgb(color)| color.map(ChannelValue::as_f32)).collect()
                            ).map(|img| image::DynamicImage::ImageRgb32F(img))
                        }
                        ColorVariant::Rgba(data) => {
                            image::ImageBuffer::from_raw(
                                width, height, data.into_iter().flat_map(|Rgba(color)| color.map(ChannelValue::as_f32)).collect()
                            ).map(|img| image::DynamicImage::ImageRgba32F(img))
                        }
                    }
                }
                ChannelVariant::U128(data) => {
                    match data {
                        ColorVariant::L(data) => {
                            image::ImageBuffer::from_raw(
                                width, height, data.into_iter().flat_map(|l| { let l = l.as_f32(); [l, l, l] }).collect()
                            ).map(|img| image::DynamicImage::ImageRgb32F(img))
                        }
                        ColorVariant::La(data) => {
                            image::ImageBuffer::from_raw(
                                width, height, data.into_iter().flat_map(|La([l, a])| { let l = l.as_f32(); [l, l, l, a.as_f32()] }).collect()
                            ).map(|img| image::DynamicImage::ImageRgba32F(img))
                        }
                        ColorVariant::Rgb(data) => {
                            image::ImageBuffer::from_raw(
                                width, height, data.into_iter().flat_map(|Rgb(color)| color.map(ChannelValue::as_f32)).collect()
                            ).map(|img| image::DynamicImage::ImageRgb32F(img))
                        }
                        ColorVariant::Rgba(data) => {
                            image::ImageBuffer::from_raw(
                                width, height, data.into_iter().flat_map(|Rgba(color)| color.map(ChannelValue::as_f32)).collect()
                            ).map(|img| image::DynamicImage::ImageRgba32F(img))
                        }
                    }
                }
                ChannelVariant::F32(data) => {
                    match data {
                        ColorVariant::L(data) => {
                            image::ImageBuffer::from_raw(
                                width, height, data.into_iter().flat_map(|l| [l, l, l]).collect()
                            ).map(|img| image::DynamicImage::ImageRgb32F(img))
                        }
                        ColorVariant::La(data) => {
                            image::ImageBuffer::from_raw(
                                width, height, data.into_iter().flat_map(|La([l, a])| [l, l, l, a]).collect()
                            ).map(|img| image::DynamicImage::ImageRgba32F(img))
                        }
                        ColorVariant::Rgb(data) => {
                            image::ImageBuffer::from_raw(
                                width, height, data.into_iter().flat_map(|Rgb(color)| color).collect()
                            ).map(|img| image::DynamicImage::ImageRgb32F(img))
                        }
                        ColorVariant::Rgba(data) => {
                            image::ImageBuffer::from_raw(
                                width, height, data.into_iter().flat_map(|Rgba(color)| color).collect()
                            ).map(|img| image::DynamicImage::ImageRgba32F(img))
                        }
                    }
                }
                ChannelVariant::F64(data) => {
                    match data {
                        ColorVariant::L(data) => {
                            image::ImageBuffer::from_raw(
                                width, height, data.into_iter().flat_map(|l| [l as f32, l as f32, l as f32]).collect()
                            ).map(|img| image::DynamicImage::ImageRgb32F(img))
                        }
                        ColorVariant::La(data) => {
                            image::ImageBuffer::from_raw(
                                width, height, data.into_iter().flat_map(|La([l, a])| [l as f32, l as f32, l as f32, a as f32]).collect()
                            ).map(|img| image::DynamicImage::ImageRgba32F(img))
                        }
                        ColorVariant::Rgb(data) => {
                            image::ImageBuffer::from_raw(
                                width, height, data.into_iter().flat_map(|Rgb(color)| color.map(|v| v as f32)).collect()
                            ).map(|img| image::DynamicImage::ImageRgb32F(img))
                        }
                        ColorVariant::Rgba(data) => {
                            image::ImageBuffer::from_raw(
                                width, height, data.into_iter().flat_map(|Rgba(color)| color.map(|v| v as f32)).collect()
                            ).map(|img| image::DynamicImage::ImageRgba32F(img))
                        }
                    }
                }
            };

            let Some(img) = img else {
                return Err(CliError::with_message(
                    CliErrorKind::ReadError,
                    "cannot convert image format"));
            };

            img.write_to(
                &mut BufWriter::new(File::create(&output)?),
                image::ImageFormat::from_path(&output).unwrap_or(image::ImageFormat::Png)
            )?;
        }
        Command::Info { files } => {
            for file in &files {
                println!("FILE: {:?}", file);

                match File::open(file) {
                    Ok(fp) => {
                        match XZIB::read(&mut BufReader::new(fp)) {
                            Ok(xzib) => {
                                print_info(&xzib);
                            }
                            Err(err) => {
                                eprintln!("Error reading file: {err}");
                            }
                        }
                    }
                    Err(err) => {
                        eprintln!("Error opening file: {err}");
                    }
                }
            }
        }
    }

    Ok(())
}

fn print_info(xzib: &XZIB) {
    let header = xzib.head();

    println!("dimensions:       {} x {}", header.width(), header.height());
    println!("number type:      {}", header.number_type());
    println!("channels:         {:3}", header.channels());
    println!("bit planes:       {:3}", header.planes());
    println!("index bit planes: {:3}", header.index_planes());
    println!("interleaved:      {}", header.is_interleaved());

    let mut chunks = Vec::with_capacity(5);
    if xzib.indx().is_some() {
        chunks.push("INDX");
    }
    if xzib.meta().is_some() {
        chunks.push("META");
    }
    if xzib.xmet().is_some() {
        chunks.push("XMET");
    }
    if xzib.body().is_some() {
        chunks.push("BODY");
    }
    if xzib.foot().is_some() {
        chunks.push("FOOT");
    }

    println!("chunks: {}", chunks.join(", "));

    if let Some(meta) = xzib.meta() {
        println!();
        println!("META:");

        if !meta.title().is_empty() {
            println!("  title: {:?}", meta.title());
        }

        if !meta.author().is_empty() {
            println!("  author:");
            for author in meta.author() {
                println!("  - {:?}", author.as_ref());
            }
        }

        if !meta.created_at().is_null() {
            println!("  created_at: {}", meta.created_at());
        }

        if !meta.license().is_empty() {
            println!("  license:");
            for license in meta.license() {
                println!("  - {:?}", license.as_ref());
            }
        }

        if !meta.links().is_empty() {
            println!("  links:");
            for links in meta.links() {
                println!("  - {:?}", links.as_ref());
            }
        }

        if !meta.comment().is_empty() {
            println!("  comment: |");
            for line in meta.comment().split("\n") {
                println!("    {line}");
            }
        }
    }

    if let Some(xmet) = xzib.xmet() {
        println!();
        println!("XMET:");

        for (key, values) in xmet.data() {
            match values.len() {
                0 => {}
                1 => {
                    let value = &values[0];
                    if value.contains("\n") {
                        println!("  {key}: |");
                        for line in value.split("\n") {
                            println!("    {line}");
                        }
                    } else {
                        println!("  {key}: {value:?}");
                    }
                }
                _ => {
                    println!("  {key}:");
                    for value in values {
                        println!("  - {value:?}");
                    }
                }
            }
        }
    }
}
