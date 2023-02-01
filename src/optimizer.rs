use std::{
    fs::File,
    io::Write,
    path::{Path, PathBuf},
};

use crate::utils::{self, ensure_parent_directory_exists};
use image::{DynamicImage, GenericImageView};

pub struct Compressor {
    quality: f32,
}

impl Compressor {
    pub fn new(quality: f32) -> Compressor {
        Compressor { quality }
    }

    pub fn set_quality(&mut self, quality: f32) {
        self.quality = quality;
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

    pub fn compress(&self) -> Vec<u8> {
        match &self.compressor {
            None => panic!("Must provide a compressor to compress the image"),
            Some(compressor) => {
                let (width, height) = self.get_img_dimensions();
                let img_as_vec = &self.img.as_bytes().to_vec();
                utils::compress(img_as_vec, width, height, compressor.quality)
                    .expect("Error in compressing")
            }
        }
    }

    fn generate_save_path(&self, w: usize) -> PathBuf {
        let path = Path::new(&self.base_path);
        let mut result = path.parent().expect("Error getting parent").to_owned();
        result.push("optimized");

        let stem = path.file_stem().expect("Error getting file name");

        let mut file_name = stem.to_os_string();

        file_name.push(format!("_{w}"));

        if let Some(compressor) = &self.compressor {
            file_name.push(format!("_{}.", compressor.quality));
        } else {
            file_name.push(".");
        }

        let ext = path
            .extension()
            .expect("Expected an extension present on img path");

        file_name.push(ext);

        result.push(file_name);
        result
    }

    fn compress_self(&self) -> Vec<u8> {
        if self.compressor.is_none() {
            panic!("Must provide a compressor");
        }
        let img = self.img.as_bytes().to_vec();
        let (src_w, src_h) = self.get_img_dimensions();

        let write_path = self.generate_save_path(src_w);

        let optimized = match &self.compressor {
            None => panic!("Must provide a compressor to compress the image"),
            Some(compressor) => utils::compress(&img, src_w, src_h, compressor.quality)
                .expect("Error in compressing"),
        };

        ensure_parent_directory_exists(&write_path);
        let mut file = File::create(write_path).expect("Error creating file");
        file.write_all(&optimized).expect("Error writing to buff");
        optimized
    }

    fn resize_and_maybe_compress(&self) {
        if self.target_sizes.is_empty() {
            panic!("Must provide at least one resize target size")
        }
        self.generate_save_path(200);
        let img = self.img.as_bytes().to_vec();
        // First, resize the image
        for (target_w, target_h) in &self.target_sizes {
            let (src_w, src_h) = self.get_img_dimensions();
            let resize_config = utils::ResizeConfig {
                src_height: src_h,
                src_width: src_w,
                dest_height: *target_h,
                dest_width: *target_w,
            };
            let optimized_img = utils::resize(&img, resize_config);
            let write_path = self.generate_save_path(*target_w);

            if let Some(compressor) = &self.compressor {
                let optimized =
                    utils::compress(&optimized_img, *target_w, *target_h, compressor.quality)
                        .expect("Error in compressing");
                ensure_parent_directory_exists(&write_path);
                let mut file = File::create(write_path).expect("Error creating file");
                file.write_all(&optimized).expect("Error writing to buff");
            } else {
                ensure_parent_directory_exists(&write_path);
                image::save_buffer(
                    write_path,
                    &optimized_img,
                    *target_w as u32,
                    *target_h as u32,
                    image::ColorType::Rgb8,
                )
                .expect("Error saving resized img");
            }
        }
    }

    pub fn optimize(&self) {
        match self.target_sizes.len() {
            0 => {
                self.compress_self();
            }
            _ => {
                self.resize_and_maybe_compress();
            }
        }
    }
}
