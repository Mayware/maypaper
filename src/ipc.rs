use tokio::sync::mpsc;

use maypaper::get_default_socket_path;
use tokio::{
    io::{AsyncBufReadExt, BufReader},
    net::UnixListener,
};
use tracing::{error, info};

use crate::event::{AcquireServer, Ipc, IpcEvent, RequestServer, TokioEvent};

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
                info!(target: "ipc", v = ?v, "Accepted listener");
                v
            },
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
                    Ok(msg) => {
                        match &msg {
                            Ipc::Set { monitor, uri } => {
                                info!(target: "ipc", "Received Set");

                                let request_server = RequestServer {
                                    path: uri.clone(),
                                    connector: monitor.clone(),
                                };
                                info!(target: "ipc", request_server = ?request_server, "Sending");
                                let _ = tx.send(TokioEvent::IpcEvent(IpcEvent::RequestServer(request_server)));
                                info!(target: "ipc", "Sent");
                            }
                            _ => {
                                //let _ = tx.send(TokioEvent::IpcEvent(IpcEvent::Ipc(msg)));
                                // TODO
                            }
                        }
                    }
                    Err(e) => error!(target: "ipc", line = %line, error = %e, "bad JSON"),
                }
            }
        });
    }
}
