use std::{error::Error, fs, path::Path};

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

pub fn resize(img: &Vec<u8>, config: ResizeConfig) -> Vec<u8> {
    let mut dst = vec![RGB::new(0, 0, 0); config.dest_width * config.dest_height];
    let mut resizer = resize::new(
        config.src_width,
        config.src_height,
        config.dest_width,
        config.dest_height,
        RGB8,
        Lanczos3,
    )
    .expect("Error creating resizer");
    println!("{:?}", &config);
    println!(
        "{} vs {}",
        dst.len(),
        config.dest_height * config.dest_width
    );

    resizer
        .resize(img.as_rgb(), &mut dst)
        .expect("error resizing image");

    let resized_image_as_u8 = dst.as_bytes();

    resized_image_as_u8.to_vec()
}

pub fn compress(
    img: &[u8],
    width: usize,
    height: usize,
    quality: f32,
) -> Result<Vec<u8>, Box<dyn Error>> {
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
            .map_err(|_| String::from("mapping data to vec failed"))?;

        Ok(jpeg_bytes)
    })
    .expect("Error in compressing the image")
}

pub fn ensure_parent_directory_exists(path: &Path) {
    if let Some(parent) = path.parent() {
        if !parent.exists() {
            fs::create_dir_all(parent).expect("Error creating parent directories");
        }
    }
}
