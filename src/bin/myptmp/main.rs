use anyhow::Result;
use clap::{Parser, Subcommand};
use std::path::Path;

use crate::common::create_dest_dir;
use crate::image::generate_image;
use crate::parallax::generate_parallax;
use crate::video::generate_video;

mod common;
mod image;
mod parallax;
mod video;

#[derive(Parser, Debug)]
#[command(
    name = "myptmp",
    version,
    about = "Generate maypaper wallpapers from templates"
)]
struct Cli {
    #[arg(long, global = true)]
    output: Option<String>,

    #[command(subcommand)]
    cmd: Cmd,
}

#[derive(Subcommand, Debug)]
enum Cmd {
    Parallax {
        #[arg(long)]
        image: String,

        #[arg(long)]
        image_depth: String,

        #[arg(long)]
        parallax_strength: Option<f32>,
    },

    Image {
        #[arg(long)]
        image: String,
    },

    Video {
        #[arg(long)]
        video: String,
    },
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    match &cli.cmd {
        Cmd::Parallax {
            image,
            image_depth,
            parallax_strength,
        } => {
            let dest_dir = create_dest_dir(cli.output.as_deref(), "parallax")?;
            generate_parallax(
                &dest_dir,
                Path::new(image),
                Path::new(image_depth),
                parallax_strength.unwrap_or(0.1),
            )?;
        }
        Cmd::Image { image } => {
            let dest_dir = create_dest_dir(cli.output.as_deref(), "image")?;
            generate_image(&dest_dir, Path::new(image))?;
        }
        Cmd::Video { video } => {
            let dest_dir = create_dest_dir(cli.output.as_deref(), "video")?;
            generate_video(&dest_dir, Path::new(video))?;
        }
    }

    Ok(())
}
