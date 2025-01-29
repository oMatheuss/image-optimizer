use image::{
    codecs::{avif::AvifEncoder, jpeg::JpegEncoder},
    imageops::FilterType,
    ImageError, ImageFormat, ImageReader,
};
use std::io::{BufReader, BufWriter, Read, Seek, Write};

pub fn process_image<R, W>(
    r: R,
    w: &mut W,
    width: u32,
    quality: u8,
    format: ImageFormat,
) -> Result<(), ImageError>
where
    R: Read + Seek,
    W: Write + Seek,
{
    let r = BufReader::new(r);
    let image = ImageReader::new(r)
        .with_guessed_format()?
        .decode()?;

    let image = image.resize(width, image.height(), FilterType::Nearest);
    let w = &mut BufWriter::new(w);

    if format == ImageFormat::Avif {
        image.write_with_encoder(AvifEncoder::new_with_speed_quality(w, 7, quality))
    } else if format == ImageFormat::Jpeg {
        image.write_with_encoder(JpegEncoder::new_with_quality(w, quality))
    } else {
        image.write_to(w, format)
    }
}