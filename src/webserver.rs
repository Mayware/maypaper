use std::collections::HashMap;

use tokio::{
    net::TcpListener,
    sync::{mpsc, oneshot},
};

use axum::{Router, routing::get};
use tower::ServiceBuilder;
use tower_http::{services::ServeDir, trace::TraceLayer};
use tracing::{debug, error, info};

use crate::event::{SetWebview, TokioEvent, WebCmd, WebEvent};

#[derive(Debug)]
struct Instance {
    url: String,
    watchers: usize,
    shutdown: oneshot::Sender<()>,
}

pub async fn web_manager(
    tx: mpsc::UnboundedSender<TokioEvent>,
    mut rx: mpsc::UnboundedReceiver<WebCmd>,
) {
    // Key is the path
    let mut instances: HashMap<String, Instance> = HashMap::new();

    while let Some(cmd) = rx.recv().await {
        debug!(target: "web", "Received a cmd from tokio");
        match cmd {
            WebCmd::AcquireServer(acquire) => {
                debug!(target: "web", acquire = ?acquire, "Received");

                if let Some(inst) = instances.get_mut(&acquire.path) {
                    inst.watchers += 1;
                    info!(target: "web", watchers = %inst.watchers, "Existing webserver found, incremented watchers");

                    let set_webview = SetWebview {
                        url: inst.url.clone(),
                        path: Some(acquire.path.clone()),
                        connector: acquire.connector,
                    };

                    debug!(target: "web", set_webview = ?set_webview, "Sending");
                    let _ = tx.send(TokioEvent::WebEvent(WebEvent::SetWebview(set_webview)));
                    debug!(target: "web", "Sent");
                    continue;
                }
                info!(target: "web", "Did not find existing webserver");

                let listener = match TcpListener::bind(("127.0.0.1", 0)).await {
                    Ok(l) => l,
                    Err(e) => {
                        error!("bind failed: {e}");
                        continue;
                    }
                };

                let port = match listener.local_addr() {
                    Ok(addr) => addr.port(),
                    Err(e) => {
                        error!("local_addr failed: {e}");
                        continue;
                    }
                };

                let url = format!("http://127.0.0.1:{port}/");

                let (shutdown_tx, shutdown_rx) = oneshot::channel::<()>();

                // A tiny race condition, I doubt it will cause any issues, if it does we'll change
                let path_for_task = acquire.path.clone();
                debug!(target: "web", path = path_for_task, "Spawning webserver task");
                tokio::spawn(async move {
                    run_web(listener, path_for_task, shutdown_rx).await;
                });

                instances.insert(
                    acquire.path.clone(),
                    Instance {
                        url: url.clone(),
                        watchers: 1,
                        shutdown: shutdown_tx,
                    },
                );
                debug!(target: "web", instances = ?instances, "Current instances");

                let set_webview = SetWebview {
                    url,
                    path: Some(acquire.path),
                    connector: acquire.connector,
                };
                debug!(target: "web", set_webview = ?set_webview, "Sending");
                let _ = tx.send(TokioEvent::WebEvent(WebEvent::SetWebview(set_webview)));
                debug!(target: "web", "Sent")
            }

            WebCmd::ReleaseServer(release) => {
                debug!(target: "web", release = ?release, "Received");

                let should_shutdown = match instances.get_mut(&release.path) {
                    Some(inst) => {
                        if inst.watchers > 1 {
                            inst.watchers -= 1;
                            info!(target: "web", path = %release.path, watchers = %inst.watchers, "Not removing, still watched");
                            false
                        } else {
                            true
                        }
                    }
                    None => {
                        error!(target: "web", "Received a release request for a path that isn't served!");
                        false
                    }
                };

                if !should_shutdown {
                    continue;
                }

                // There are no longer any watchers
                if let Some(inst) = instances.remove(&release.path) {
                    info!(target: "web", path = %release.path, "No watchers, attempting to shutdown");
                    let _ = inst.shutdown.send(());
                }
            }
        }
    }
}

async fn run_web(listener: TcpListener, path: String, shutdown: oneshot::Receiver<()>) {
    let addr = listener.local_addr().unwrap();
    info!(target: "web", _path = %path, %addr, "Starting webserver");

    let static_site = ServeDir::new(path.clone()).append_index_html_on_directories(true);

    let app = Router::new()
        .route("/api/health", get(|| async { "ok" }))
        .fallback_service(static_site)
        .layer(ServiceBuilder::new().layer(TraceLayer::new_for_http()));

    let path_for_shutdown = path.clone();

    let server = axum::serve(listener, app).with_graceful_shutdown(async move {
        let _ = shutdown.await;
        info!(target: "web", path = %path_for_shutdown, "Shutdown webserver");
    });

    if let Err(e) = server.await {
        error!(target: "web", path = %path, error = %e, "Web server error");
    }
}
