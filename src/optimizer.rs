use std::{
    fs::File,
    io::Write,
    path::{Path, PathBuf},
};

use crate::utils::{self, ensure_parent_directory_exists};
use anyhow::{anyhow, Ok};
use clap::ValueEnum;
use image::{DynamicImage, GenericImageView};

#[derive(Debug, ValueEnum, Clone)]
pub enum Encoder {
    WebP,
    MozJpeg,
}

pub struct Compressor {
    quality: f32,
    encoder: Encoder,
}

impl Compressor {
    pub fn new(quality: f32) -> Compressor {
        Compressor {
            quality,
            encoder: Encoder::MozJpeg,
        }
    }

    pub fn set_quality(&mut self, quality: f32) {
        self.quality = quality;
    }

    pub fn set_encoder(&mut self, encoder: Encoder) {
        self.encoder = encoder;
    }
}

pub struct Optimizer {
    img: DynamicImage,
    base_path: String,
    target_sizes: Vec<(usize, usize)>,
    compressor: Option<Compressor>,
}

impl Optimizer {
    pub fn new(img: DynamicImage, img_path: &str) -> Optimizer {
        Optimizer {
            img,
            base_path: img_path.to_string(),
            target_sizes: vec![],
            compressor: None,
        }
    }

    pub fn set_encoder(&mut self, encoder: Encoder) {
        match &mut self.compressor {
            None => {
                let mut compressor = Compressor::new(75.0);
                compressor.set_encoder(encoder);
                self.compressor = Some(compressor);
            }
            Some(compressor) => compressor.set_encoder(encoder),
        }
    }

    pub fn set_quality(&mut self, quality: f32) {
        match &mut self.compressor {
            None => self.compressor = Some(Compressor::new(quality)),
            Some(compressor) => compressor.set_quality(quality),
        }
    }

    pub fn set_targets(&mut self, target_sizes: Vec<(usize, usize)>) {
        self.target_sizes = target_sizes;
    }

    pub fn add_target(&mut self, target: (usize, usize)) {
        self.target_sizes.push(target);
    }

    fn get_img_dimensions(&self) -> (usize, usize) {
        let (w, h) = self.img.dimensions();
        (w.try_into().unwrap(), h.try_into().unwrap())
    }

    pub fn compress(&self) -> anyhow::Result<Vec<u8>> {
        match &self.compressor {
            None => Err(anyhow!(
                "Must provide a quality value/compressor to compress an image"
            )),
            Some(compressor) => {
                let (width, height) = self.get_img_dimensions();
                let img_as_vec = &self.img.as_bytes().to_vec();
                match &compressor.encoder {
                    Encoder::WebP => utils::compress_webp(
                        img_as_vec,
                        width as u32,
                        height as u32,
                        compressor.quality,
                    ),
                    Encoder::MozJpeg => {
                        utils::compress_mozjpeg(img_as_vec, width, height, compressor.quality)
                    }
                }
            }
        }
    }

    fn generate_save_path(&self, w: usize) -> anyhow::Result<PathBuf> {
        let path = Path::new(&self.base_path);
        let mut result = path
            .parent()
            .ok_or(anyhow!("Provided image must have a parent directory"))?
            .to_owned();
        result.push("optimized");

        let stem = path.file_stem().ok_or(anyhow!("Error getting file name"))?;

        let mut file_name = stem.to_os_string();

        file_name.push(format!("_{w}"));

        if let Some(compressor) = &self.compressor {
            file_name.push(format!("_{}.", compressor.quality));
            match compressor.encoder {
                Encoder::MozJpeg => {
                    let ext = path
                        .extension()
                        .ok_or(anyhow!("Expected an extension present on image path"))?;
                    file_name.push(ext);
                }
                Encoder::WebP => file_name.push("webp"),
            }
        } else {
            file_name.push(".");
            let ext = path
                .extension()
                .ok_or(anyhow!("Expected an extension present on image path"))?;
            file_name.push(ext);
        }

        result.push(file_name);
        Ok(result)
    }

    fn compress_self(&self) -> anyhow::Result<()> {
        let compressor = match &self.compressor {
            None => Err(anyhow!(
                "Must provide a quality value/compressor to compress an image"
            )),
            Some(compressor) => Ok(compressor),
        }?;
        let img = self.img.as_bytes().to_vec();
        let (src_w, src_h) = self.get_img_dimensions();

        let write_path = self.generate_save_path(src_w)?;

        let optimized = match compressor.encoder {
            Encoder::WebP => {
                utils::compress_webp(&img, src_w as u32, src_h as u32, compressor.quality)
            }
            Encoder::MozJpeg => utils::compress_mozjpeg(&img, src_w, src_h, compressor.quality),
        }?;

        ensure_parent_directory_exists(&write_path)?;
        let mut file = File::create(write_path)?;
        file.write_all(&optimized)?;
        Ok(())
    }

    fn resize_and_maybe_compress(&self) -> anyhow::Result<()> {
        if self.target_sizes.is_empty() {
            return Err(anyhow!("Must provide at least one resize target size"));
        }
        let img = self.img.as_bytes().to_vec();
        // First, resize the image
        let (src_w, src_h) = self.get_img_dimensions();
        for (target_w, target_h) in &self.target_sizes {
            let resize_config = utils::ResizeConfig {
                src_height: src_h,
                src_width: src_w,
                dest_height: *target_h,
                dest_width: *target_w,
            };

            let write_path = self.generate_save_path(*target_w)?;

            let resized_img = utils::resize(&img, resize_config)?;

            if let Some(compressor) = &self.compressor {
                let optimized = match compressor.encoder {
                    Encoder::WebP => utils::compress_webp(
                        &resized_img,
                        *target_w as u32,
                        *target_h as u32,
                        compressor.quality,
                    ),
                    Encoder::MozJpeg => utils::compress_mozjpeg(
                        &resized_img,
                        *target_w,
                        *target_h,
                        compressor.quality,
                    ),
                }?;
                ensure_parent_directory_exists(&write_path)?;
                let mut file = File::create(write_path)?;
                file.write_all(&optimized)?;
            } else {
                ensure_parent_directory_exists(&write_path)?;
                image::save_buffer(
                    write_path,
                    &resized_img,
                    *target_w as u32,
                    *target_h as u32,
                    image::ColorType::Rgb8,
                )?;
            }
        }
        Ok(())
    }

    pub fn optimize(&self) -> anyhow::Result<()> {
        match self.target_sizes.len() {
            0 => self.compress_self(),
            _ => self.resize_and_maybe_compress(),
        }
    }
}
