use maypaper::get_default_socket_path;
use tokio::sync::mpsc;

use tokio::{
    io::{AsyncBufReadExt, BufReader},
    net::UnixListener,
};
use tracing::{debug, error, info};

use crate::event::{Ipc, IpcEvent, RequestServer, RequestWebview, TokioEvent};

pub async fn ipc_server(tx: mpsc::UnboundedSender<TokioEvent>) {
    let socket_path = get_default_socket_path();
    let _ = std::fs::remove_file(&socket_path);

    let listener = match UnixListener::bind(&socket_path) {
        Ok(l) => l,
        Err(e) => {
            error!(target: "ipc", socket_path = ?socket_path, error = %e, "Failed to bind socket");
            return;
        }
    };

    info!(target: "ipc", socket_path = ?socket_path, "Listening");

    loop {
        let (stream, _addr) = match listener.accept().await {
            Ok(v) => {
                debug!(target: "ipc", v = ?v, "Accepted listener");
                v
            }
            Err(e) => {
                error!(target: "ipc", error = %e, "Accept error");
                continue;
            }
        };

        let tx = tx.clone();
        tokio::spawn(async move {
            let mut lines = BufReader::new(stream).lines();

            while let Ok(Some(line)) = lines.next_line().await {
                let line = line.trim();
                if line.is_empty() {
                    continue;
                }

                match serde_json::from_str::<Ipc>(line) {
                    Ok(msg) => match &msg {
                        Ipc::SetPath { monitor, path } => {
                            info!(target: "ipc", "Received SetPath");

                            let request_server = RequestServer {
                                path: path.clone(),
                                connector: monitor.clone(),
                            };
                            debug!(target: "ipc", request_server = ?request_server, "Sending");
                            let _ = tx.send(TokioEvent::IpcEvent(IpcEvent::RequestServer(
                                request_server,
                            )));
                            debug!(target: "ipc", "Sent");
                        }

                        Ipc::SetUrl { monitor, url } => {
                            info!(target: "ipc", "Received SetUrl");

                            let request_webview = RequestWebview {
                                url: url.clone(),
                                connector: monitor.clone(),
                            };

                            debug!(target: "ipc", request_webview = ?request_webview, "Sending");
                            let _ = tx.send(TokioEvent::IpcEvent(IpcEvent::RequestWebview(
                                request_webview,
                            )));
                            debug!(target: "ipc", "Sent");
                        }
                    },
                    Err(e) => error!(target: "ipc", line = %line, error = %e, "bad JSON"),
                }
            }
        });
    }
}
