use anyhow::{Context, Result, bail, ensure};
use clap::{Parser, Subcommand};
use console::style;
use maypaper::Paths;
use serde::Deserialize;
use std::{
    env, fs,
    path::{Path, PathBuf},
    process::{Command, Stdio},
    thread,
    time::{Duration, Instant},
};
use tracing::{error, info, warn};
use tracing_subscriber::EnvFilter;

static DEFAULT_CONFIG: &str = r#"# maypaper wallpapers.toml
#
# Each entry installs/updates a wallpaper into:
#   ~/.config/maypaper/wallpapers/<name>/
#
# Notes:
# - "name" becomes the folder name. Avoid slashes.
# - "subdir" is optional. Use it when the wallpaper lives in a monorepo.
# - The installer uses shallow + sparse checkout to avoid downloading huge repos.
#
# Example:
# [[wallpapers]]
# name = "Aurora Waves"
# repo = "https://github.com/example/maypaper-wallpapers.git"
# subdir = "wallpapers/aurora-waves"

[[wallpapers]]
name = "Example Wallpaper"
repo = "https://github.com/example/maypaper-wallpapers.git"
subdir = "wallpapers/example"
"#;

#[derive(Parser, Debug)]
#[command(
    name = "myppm",
    version,
    about = "Maypaper package manager: manages website wallpapers from git repos",
    arg_required_else_help = true
)]
struct Cli {
    #[arg(
        long,
        help = "The config directory. If left unspecified, XDG_CONFIG_HOME/maypaper is used"
    )]
    config_dir: Option<PathBuf>,

    #[command(subcommand)]
    cmd: Cmd,
}

#[derive(Subcommand, Debug)]
enum Cmd {
    /// Update wallpapers, as specified, and clean up any wallpapers that are no longer specified
    Sync {
        #[arg(
            long,
            short,
            help = "The name of the wallpaper you want to update. If left unspecified, all wallpapers are updated"
        )]
        name: Option<String>,
    },

    /// Output wallpapers recognised by the program. Useful for debugging issues
    List,
}

#[derive(Debug, Deserialize)]
struct Configuration {
    #[serde(default)]
    wallpapers: Vec<Wallpaper>,
}

#[derive(Debug, Deserialize, Clone)]
struct Wallpaper {
    name: String,
    repo: String,
    #[serde(default)]
    subdir: Option<String>,
}

fn ensure_default_config(path: &Paths) -> Result<()> {
    if path.config.exists() {
        return Ok(());
    }

    let default = DEFAULT_CONFIG;
    fs::write(&path.config, default.as_bytes())
        .with_context(|| format!("Failed to write default config {}", path.config.display()))?;
    info!(path = %path.config.display(), "Wrote default config");

    Ok(())
}

fn load_config(path: &Path) -> Result<Configuration> {
    let config_string =
        fs::read_to_string(path).with_context(|| format!("Failed to read {}", path.display()))?;
    let config: Configuration = toml::from_str(&config_string)
        .with_context(|| format!("Failed to parse {}", path.display()))?;

    for (i, wallpaper) in config.wallpapers.iter().enumerate() {
        if wallpaper.name.is_empty() {
            bail!("Wallpapers[{i}].name is empty");
        }

        validate_wallpaper_name(&wallpaper.name)
            .with_context(|| format!("Wallpapers[{i}].name is not safe as a folder name"))?;

        if wallpaper.repo.is_empty() {
            bail!("Wallpapers[{i}].repo is empty");
        }

        // Note that the subdir field is optional
    }

    Ok(config)
}

fn sync_wallpaper(paths: &Paths, wallpaper: &Wallpaper) -> Result<()> {
    let destination = paths.wallpapers.join(&wallpaper.name);

    if !destination.exists() {
        println!("{} {}", style("Installing:").cyan(), wallpaper.name);
        clone_sparse_repo(&destination, &wallpaper.repo, wallpaper.subdir.as_deref())?;
        return Ok(());
    }

    println!("{} {}", style("Updating:").cyan(), wallpaper.name);

    // Incase the user updated the subdir
    git(&destination, &["sparse-checkout", "init", "--cone"])?;
    if let Some(subdir) = wallpaper.subdir.as_deref() {
        git(&destination, &["sparse-checkout", "set", subdir])?;
    }

    // Avoid overwriting changes
    if is_dirty(&destination)? {
        warn!(
            "Local changes detected! Please stash your changes! Skipping: {}",
            wallpaper.name
        );
        return Ok(());
    }

    ensure_branch(&destination)?;

    git(&destination, &["pull", "--ff-only"])?;

    Ok(())
}

fn clone_sparse_repo(destination: &Path, repo_url: &str, subdir: Option<&str>) -> Result<()> {
    let out = Command::new("git")
        .args([
            "clone",
            "--depth=1",
            "--single-branch",
            "--filter=blob:none",
            "--no-checkout",
            repo_url,
        ])
        .arg(destination)
        .output()
        .context("Git clone failed!")?;

    ensure!(
        out.status.success(),
        "git clone for {} failed!",
        destination.display()
    );

    git(destination, &["sparse-checkout", "init", "--cone"])?;

    // Set the subdir, if specified
    if let Some(subdir) = subdir {
        git(destination, &["sparse-checkout", "set", subdir])?;
    }

    // Actually get the stuff from remote
    git(destination, &["checkout", "-f"])?;
    ensure_branch(destination)?;
    Ok(())
}

fn ensure_branch(repo_dir: &Path) -> Result<()> {
    // Get upstream
    let default_remote = git_capture(
        repo_dir,
        &[
            "symbolic-ref",
            "--quiet",
            "--short",
            "refs/remotes/origin/HEAD",
        ],
    )
    .unwrap_or_else(|_| "origin/master".to_string());

    // Ensure we're on a branch, and the same one each time
    git(
        repo_dir,
        &["checkout", "-B", "mayppm", "--track", default_remote.trim()],
    )?;
    Ok(())
}

fn is_dirty(repo_dir: &Path) -> Result<bool> {
    let out = git_capture(repo_dir, &["status", "--porcelain"])?;
    Ok(!out.trim().is_empty())
}

fn validate_wallpaper_name(name: &str) -> Result<()> {
    if name.contains('/') || name.contains('\\') {
        bail!("Name contains a path separator ('/' or '\\')");
    }
    if name == "." || name == ".." || name.contains("..") {
        bail!("Name contains '.' or '..' segments");
    }
    Ok(())
}

fn git(cwd: &Path, args: &[&str]) -> Result<()> {
    println!("Executing: git {:?}", args);
    let out = Command::new("git")
        .current_dir(cwd)
        .args(args)
        .status()
        .with_context(|| format!("Failed to run git {:?} in {}", args, cwd.display()))?;

    ensure!(out.success(), "git {:?} failed in {}", args, cwd.display());
    Ok(())
}

fn git_capture(cwd: &Path, args: &[&str]) -> Result<String> {
    let out = Command::new("git")
        .current_dir(cwd)
        .args(args)
        .output()
        .with_context(|| format!("git {:?} failed in {}", args, cwd.display()))?;

    if !out.status.success() {
        bail!(
            "git {:?} failed: {}",
            args,
            String::from_utf8_lossy(&out.stderr)
        );
    }

    Ok(String::from_utf8_lossy(&out.stdout).trim().to_string())
}

fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .init();

    let cli = Cli::parse();
    let paths = Paths::get_dirs(cli.config_dir)?;
    paths.ensure_dirs()?;
    ensure_default_config(&paths)?;

    match cli.cmd {
        Cmd::List => {
            let config = load_config(&paths.config)?;
            if config.wallpapers.is_empty() {
                println!("{}", style("No wallpapers configured").yellow());
            } else {
                for (i, wallpaper) in config.wallpapers.iter().enumerate() {
                    println!("[{}] {:#?}", style(i).cyan(), wallpaper);
                }
            }
            Ok(())
        }
        Cmd::Sync { name } => {
            let config = load_config(&paths.config)?;
            info!(count = config.wallpapers.len(), "Loaded wallpapers.toml");

            if config.wallpapers.is_empty() {
                warn!(
                    "No wallpapers configured! Edit wallpapers.toml in {}!",
                    paths.config.display()
                );
                return Ok(());
            }

            /* Update / Install */
            if let Some(target) = name {
                // Update just the target
                let Some(wallpaper) = config
                    .wallpapers
                    .iter()
                    .find(|wallpaper| wallpaper.name == target)
                else {
                    bail!("wallpaper not found in config: {}", target);
                };

                match sync_wallpaper(&paths, wallpaper) {
                    Ok(()) => println!("{} {}", style("Synced:").green(), wallpaper.name),
                    Err(e) => bail!("Sync failed: {}, {}", wallpaper.name, e),
                }
                return Ok(());
            }

            // Update all
            for wallpaper in &config.wallpapers {
                match sync_wallpaper(&paths, wallpaper) {
                    Ok(()) => println!("{} {}", style("Synced:").green(), wallpaper.name),
                    Err(e) => error!("Sync failed: {}, {}", wallpaper.name, e),
                }
            }

            /* Remove no longer required wallpapers */
            let entries: Vec<PathBuf> = fs::read_dir(paths.wallpapers)?
                .filter_map(Result::ok)
                .map(|e| e.path())
                .collect();

            for entry in entries {
                let name = entry.file_name().unwrap().to_str().unwrap();

                let mut found = false;
                for wallpaper in &config.wallpapers {
                    if wallpaper.name == name {
                        found = true;
                        break;
                    }
                }

                if found {
                    continue;
                }

                if entry.is_dir() {
                    fs::remove_dir_all(&entry)?;
                } else {
                    // Although there shouldn't be any files here,
                    // do it anyway
                    fs::remove_file(&entry)?;
                }
                println!("{} {}", style("Removed:").green(), name);
            }

            Ok(())
        }
    }
}
