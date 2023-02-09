use anyhow::anyhow;
use std::{fs, io, path::Path};

use resize::px::RGB;
use resize::Pixel::RGB8;
use resize::Type::Lanczos3;
use rgb::{ComponentBytes, FromSlice};

#[derive(Debug)]
pub struct ResizeConfig {
    pub src_height: usize,
    pub src_width: usize,
    pub dest_height: usize,
    pub dest_width: usize,
}

pub fn resize(img: &Vec<u8>, config: ResizeConfig) -> anyhow::Result<Vec<u8>> {
    let mut dst = vec![RGB::new(0, 0, 0); config.dest_width * config.dest_height];
    let mut resizer = resize::new(
        config.src_width,
        config.src_height,
        config.dest_width,
        config.dest_height,
        RGB8,
        Lanczos3,
    )
    .map_err(|_| anyhow!("Error creating resizer"))?;

    resizer
        .resize(img.as_rgb(), &mut dst)
        .map_err(|_| anyhow!("Error resizing image"))?;

    let resized_image_as_u8 = dst.as_bytes();

    Ok(resized_image_as_u8.to_vec())
}

pub fn compress_mozjpeg(
    img: &[u8],
    width: usize,
    height: usize,
    quality: f32,
) -> Result<Vec<u8>, anyhow::Error> {
    std::panic::catch_unwind(|| {
        let mut comp = mozjpeg::Compress::new(mozjpeg::ColorSpace::JCS_RGB);

        comp.set_size(width, height);
        comp.set_mem_dest();
        comp.set_quality(quality);
        comp.start_compress();

        assert!(comp.write_scanlines(img));

        comp.finish_compress();
        let jpeg_bytes = comp
            .data_to_vec()
            .map_err(|_| anyhow!("Image compression failed"))?;

        Ok(jpeg_bytes)
    })
    .map_err(|_| anyhow!("Error compressing image"))?
}

pub fn ensure_parent_directory_exists(path: &Path) -> io::Result<()> {
    if let Some(parent) = path.parent() {
        if !parent.exists() {
            fs::create_dir_all(parent)?;
        }
    }
    Ok(())
}

pub fn compress_webp(
    img: &[u8],
    width: u32,
    height: u32,
    quality: f32,
) -> Result<Vec<u8>, anyhow::Error> {
    let encoder = webp::Encoder::from_rgb(img, width, height);
    let encoded_img = (*encoder.encode(quality)).to_vec();
    Ok(encoded_img)
}
