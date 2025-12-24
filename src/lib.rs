use anyhow::Result;
use std::{fs, path::PathBuf};

use tracing::error;

pub mod event;


pub fn get_default_socket_path() -> PathBuf {
    if let Some(dir) = env::var_os("XDG_RUNTIME_DIR") {
        PathBuf::from(dir).join("maypaper.sock")
    } else {
        error!("XDG_RUNTIME_DIR is not set — specify --socket manually");
        std::process::exit(1);
    }
}

#[derive(Debug, Clone)]
pub struct Paths {
    pub base: PathBuf,       // ~/.config/maypaper (or override)
    pub config: PathBuf,     // ~/.config/maypaper/wallpapers.toml
    pub wallpapers: PathBuf, // ~/.config/maypaper/wallpapers/
}

impl Paths {
    pub fn get_dirs(config_dir_override: Option<PathBuf>) -> Result<Self> {
        let base = match config_dir_override {
            Some(p) => p,
            None => PathBuf::from(env::var("XDG_CONFIG_HOME")?).join("maypaper"),
        };

        Ok(Self {
            config: base.join("wallpapers.toml"),
            wallpapers: base.join("wallpapers"),
            base,
        })
    }

    pub fn ensure_dirs(&self) -> Result<()> {
        fs::create_dir_all(&self.base)?;
        fs::create_dir_all(&self.wallpapers)?;
        Ok(())
    }
}
