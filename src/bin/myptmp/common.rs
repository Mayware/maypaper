use anyhow::{Context, Result, bail};
use rust_embed::Embed;
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Embed)]
#[folder = "templates/"]
pub struct Templates;

pub fn create_dest_dir(path: Option<&str>, default_name: &str) -> Result<PathBuf> {
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

pub fn copy_file_into_dir(dest_dir: &Path, src: &Path, label: &str) -> Result<String> {
    let name = src
        .file_name()
        .and_then(|s| s.to_str())
        .with_context(|| format!("Invalid {label} filename: {}", src.display()))?
        .to_string();

    fs::copy(src, dest_dir.join(&name)).with_context(|| {
        format!(
            "Failed to copy {label} {} -> {}",
            src.display(),
            dest_dir.display()
        )
    })?;

    Ok(name)
}

pub fn materialize_template_subtree(
    dest_dir: &Path,
    prefix: &str,
    mut rewrite_index: impl FnMut(String) -> Result<String>,
) -> Result<()> {
    // Be forgiving: allow "parallax" or "parallax/"
    let prefix = if prefix.ends_with('/') {
        prefix.to_string()
    } else {
        format!("{prefix}/")
    };

    for asset_path in Templates::iter() {
        let asset_path = asset_path.as_ref();

        // Only take this subtree
        let Some(rel) = asset_path.strip_prefix(&prefix) else {
            continue;
        };

        let Some(asset) = Templates::get(asset_path) else {
            continue;
        };

        let out_path: PathBuf = dest_dir.join(rel);

        if let Some(parent) = out_path.parent() {
            fs::create_dir_all(parent)
                .with_context(|| format!("Failed to create dir: {}", parent.display()))?;
        }

        if rel == "index.html" {
            let html = std::str::from_utf8(&asset.data)
                .with_context(|| format!("templates/{prefix}index.html is not valid UTF-8"))?
                .to_string();

            let html = rewrite_index(html)?;
            fs::write(&out_path, html.as_bytes())
                .with_context(|| format!("Failed to write {}", out_path.display()))?;
        } else {
            fs::write(&out_path, asset.data.as_ref())
                .with_context(|| format!("Failed to write {}", out_path.display()))?;
        }
    }

    Ok(())
}

