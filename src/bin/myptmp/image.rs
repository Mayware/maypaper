use anyhow::Result;
use std::path::Path;

use crate::common::{copy_file_into_dir, materialize_template_subtree};

pub fn generate_image(dest_dir: &Path, image: &Path) -> Result<()> {
    let image_name = copy_file_into_dir(dest_dir, image, "image")?;

    materialize_template_subtree(dest_dir, "image", |mut html| {
        html = html.replace("{IMAGE}", &image_name);
        Ok(html)
    })
}
