use std::vec;

use anyhow::anyhow;
use clap::Parser;
use image::{self, GenericImageView};
mod optimizer;
mod utils;
use optimizer::{Encoder, Optimizer};

#[derive(Debug, Parser)]
struct Args {
    img_src: String,
    #[arg(long, short)]
    widths: Option<Vec<usize>>,
    #[arg(long, short)]
    quality: Option<f32>,
    #[arg(long, short)]
    encoder: Option<Encoder>,
}

fn compute_height_preserving_aspect_ratio(
    img_dimensions: (usize, usize),
    target_width: usize,
) -> usize {
    let (w, h) = img_dimensions;
    let factor = w / target_width;
    h / factor
}

fn main() -> anyhow::Result<()> {
    let args = Args::parse();
    let img = image::open(&args.img_src)?;
    let dimensions = img.dimensions();

    let mut optimizer = Optimizer::new(img, &args.img_src);

    if args.widths.is_none() && args.quality.is_none() {
        return Err(anyhow!("Either widths or quality must be provided"));
    }

    if let Some(target_widths) = args.widths {
        let w: usize = dimensions.0.try_into().unwrap();
        let h: usize = dimensions.1.try_into().unwrap();
        let mut computed_target_dimensions = vec![];
        for target_width in target_widths {
            let target_height = compute_height_preserving_aspect_ratio((w, h), target_width);
            computed_target_dimensions.push((target_width, target_height));
        }
        optimizer.set_targets(computed_target_dimensions);
    }

    if let Some(quality) = args.quality {
        optimizer.set_quality(quality);
    }

    if let Some(encoder) = args.encoder {
        optimizer.set_encoder(encoder);
    }

    optimizer.optimize()
}
