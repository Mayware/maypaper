use anyhow::Result;
use std::path::Path;

use crate::common::{copy_file_into_dir, materialize_template_subtree};

pub fn generate_video(dest_dir: &Path, video: &Path) -> Result<()> {
    let video_name = copy_file_into_dir(dest_dir, video, "video")?;

    materialize_template_subtree(dest_dir, "video", |mut html| {
        html = html.replace("{VIDEO}", &video_name);
        Ok(html)
    })
}
