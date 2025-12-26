use anyhow::Result;
use std::path::Path;

use crate::common::{copy_file_into_dir, materialize_template_subtree};

pub fn generate_parallax(
    dest_dir: &Path,
    image: &Path,
    depth: &Path,
    parallax_strength: f32,
) -> Result<()> {
    let image_name = copy_file_into_dir(dest_dir, image, "image")?;
    let depth_name = copy_file_into_dir(dest_dir, depth, "depth image")?;

    materialize_template_subtree(dest_dir, "parallax", |mut html| {
        html = html.replace("{IMAGE}", &image_name);
        html = html.replace("{IMAGE_DEPTH}", &depth_name);
        html = html.replace("{PARALLAX_STRENGTH}", &parallax_strength.to_string());
        Ok(html)
    })
}

