use std::io::{self, Write};
use std::os::unix::net::UnixStream;
use std::path::PathBuf;
use anyhow::Result;

use clap::{Parser, Subcommand};
use maypaper::{Ipc, Paths, get_default_socket_path};
use tracing::error;

#[derive(Parser, Debug)]
#[command(name = "mypctl", version, about = "Control maypaper via IPC")]
struct Cli {
    /// The config directory. If left unspecified, XDG_CONFIG_HOME/maypaper is used
    #[arg(long)]
    config_dir: Option<PathBuf>,

    /// Path to the maypaper IPC socket (defaults to /run/user/<uid>/maypaper.sock)
    #[arg(long, value_name = "PATH")]
    socket: Option<PathBuf>,

    #[command(subcommand)]
    cmd: Cmd,
}

#[derive(Subcommand, Debug)]
enum Cmd {
    Set {
        #[arg(long)]
        monitor: Option<usize>,

        #[arg(long, conflicts_with = "name", required = true)]
        uri: Option<String>,

        #[arg(long, conflicts_with = "uri", required = true)]
        name: Option<String>,
    },
    Reload {
        #[arg(long)]
        monitor: Option<usize>,
    },
}

fn send_msg(socket_path: &PathBuf, msg: &Ipc) -> io::Result<()> {
    let mut stream = UnixStream::connect(socket_path)?;

    // Delimiter is newline
    let line =
        serde_json::to_string(msg).map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;

    stream.write_all(line.as_bytes())?;
    stream.write_all(b"\n")?;
    stream.flush()?;
    Ok(())
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    let paths = Paths::get_dirs(cli.config_dir)?;
    paths.ensure_dirs()?;
    let socket_path = cli.socket.unwrap_or_else(get_default_socket_path);

    let msg = match cli.cmd {
        Cmd::Set { monitor, uri, name } => Ipc::Set {
            monitor,
            uri: if uri.is_some() {
                uri.unwrap()
            } else {
                format!("file://{}/{}/index.html", paths.wallpapers.display(), name.unwrap())
            },
        },
        Cmd::Reload { monitor } => Ipc::Reload { monitor },
    };

    if let Err(e) = send_msg(&socket_path, &msg) {
        error!(
            "mypctl: failed to send command to socket {:?}: {}",
            socket_path, e
        );
        std::process::exit(1);
    }

    Ok(())
}
