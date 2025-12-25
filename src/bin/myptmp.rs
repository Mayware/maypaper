use anyhow::{Context, Result, bail};
use clap::{Parser, Subcommand};
use rust_embed::Embed;
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Embed)]
#[folder = "templates/"]
struct Templates;

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
}

fn create_dest_dir(path: Option<&str>, default_name: &str) -> Result<PathBuf> {
    let final_path = match path {
        None => PathBuf::from(".").join(default_name),
        Some(s) => {
            let has_sep = s.contains('/') || s.contains('\\');
            let ends_with_sep = s.ends_with('/') || s.ends_with('\\');

            let dest_dir = if !has_sep {
                // Bare name means CWD
                PathBuf::from(".").join(s)
            } else {
                let p = PathBuf::from(s);
                if ends_with_sep {
                    // Assume they mean just put in that dir
                    p.join(default_name)
                } else {
                    // Use their path
                    p
                }
            };
            dest_dir
        }
    };

    // Dont overwrite existing
    if final_path.exists() {
        bail!("Destination already exists: {}", final_path.display());
    }

    fs::create_dir_all(&final_path)
        .with_context(|| format!("Failed to create destination dir: {}", final_path.display()))?;

    Ok(final_path)
}

fn generate_parallax(
    dest_dir: &Path,
    image: &Path,
    depth: &Path,
    parallax_strength: f32,
) -> Result<()> {
    // Copy the two images into the generated folder (root).
    let image_name = image
        .file_name()
        .and_then(|s| s.to_str())
        .with_context(|| format!("Invalid image filename: {}", image.display()))?;

    let depth_name = depth
        .file_name()
        .and_then(|s| s.to_str())
        .with_context(|| format!("Invalid depth image filename: {}", depth.display()))?;

    fs::copy(image, dest_dir.join(image_name)).with_context(|| {
        format!(
            "Failed to copy image {} -> {}",
            image.display(),
            dest_dir.display()
        )
    })?;

    fs::copy(depth, dest_dir.join(depth_name)).with_context(|| {
        format!(
            "Failed to copy depth image {} -> {}",
            depth.display(),
            dest_dir.display()
        )
    })?;

    for asset_path in Templates::iter() {
        let asset_path = asset_path.as_ref();

        // Only take the parallax subtree
        let Some(rel) = asset_path.strip_prefix("parallax/") else {
            continue;
        };

        let Some(asset) = Templates::get(asset_path) else {
            continue;
        };

        let out_path = dest_dir.join(rel);

        if let Some(parent) = out_path.parent() {
            fs::create_dir_all(parent)
                .with_context(|| format!("Failed to create dir: {}", parent.display()))?;
        }

        if rel == "index.html" {
            let mut html = std::str::from_utf8(&asset.data)
                .context("templates/parallax/index.html is not valid UTF-8")?
                .to_string();

            html = html.replace("{IMAGE}", image_name);
            html = html.replace("{IMAGE_DEPTH}", depth_name);
            html = html.replace("{PARALLAX_STRENGTH}", &(parallax_strength).to_string());

            fs::write(&out_path, html.as_bytes())
                .with_context(|| format!("Failed to write {}", out_path.display()))?;
        } else {
            fs::write(&out_path, asset.data.as_ref())
                .with_context(|| format!("Failed to write {}", out_path.display()))?;
        }
    }

    Ok(())
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
    }

    Ok(())
}
