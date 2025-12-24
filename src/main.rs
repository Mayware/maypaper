use gtk4::glib::object::Cast;
use gtk4::prelude::{ApplicationExt, ApplicationExtManual};
use gtk4::{Application, glib};

use gtk4::gdk::prelude::{DisplayExt, MonitorExt};
use gtk4::gio::prelude::ListModelExt;
use gtk4::{gdk, gio};

use tokio::sync::mpsc;
use tracing::info;
use webkit6::WebView;
use webkit6::prelude::WebViewExt;

use crate::event::{IpcEvent, ReleaseServer, TokioEvent, UiCmd, UiEvent, WebCmd, WebEvent};

mod event;
mod ipc;
mod webserver;
mod webview;

fn start_tokio(ui_tx: async_channel::Sender<UiCmd>, bg_rx: async_channel::Receiver<UiEvent>) {
    std::thread::spawn(move || {
        let rt = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .expect("failed to build tokio runtime");

        rt.block_on(async move {
            let (tokio_tx, mut tokio_rx) = mpsc::unbounded_channel::<TokioEvent>();

            //let (ipc_tx, ipc_rx) = mpsc::unbounded_channel();
            let (web_tx, web_rx) = mpsc::unbounded_channel::<WebCmd>();

            // Each spawned task can take: tokio_tx, their_own_rx, if they need it
            tokio::spawn(ipc::ipc_server(tokio_tx.clone()));
            info!(target: "tokio", "Started ipc_server");
            tokio::spawn(webserver::web_manager(tokio_tx.clone(), web_rx));
            info!(target: "tokio", "Started web_manager");

            loop {
                tokio::select! {
                    bg = bg_rx.recv() => {
                        info!(target: "tokio", "Received an event from glib");
                        match bg {
                            Ok(event) => {
                                match event {
                                    UiEvent::ReleaseServer(release_server) => {
                                        info!(target: "tokio", release_server = ?release_server, "Received");
                                        info!(target: "tokio", "Forwarding to webserver");
                                        let _ = web_tx.send(WebCmd::ReleaseServer(release_server));
                                    },
                                }
                            }
                            Err(_) => {}
                        }
                    }

                    tk = tokio_rx.recv() => {
                        match tk {
                            Some(event) => {
                                match event {
                                    TokioEvent::IpcEvent(ipc_event) => match ipc_event {
                                        IpcEvent::AcquireServer(acquire_server) => {
                                            info!(target: "tokio", acquire_server = ?acquire_server, "Received");
                                            info!(target: "tokio", "Forwarding to webserver");
                                            let _ = web_tx.send(WebCmd::AcquireServer(acquire_server));
                                        }
                                        IpcEvent::ReloadWebview(_reload_webview) => todo!(),
                                    },
                                    TokioEvent::WebEvent(web_event) => match web_event {
                                        WebEvent::SetWebview(set_webview) => {
                                            info!(target: "tokio", set_webview = ?set_webview, "Received");
                                            info!(target: "tokio", "Forwarding to glib");
                                            let _ = ui_tx.send(UiCmd::SetWebview(set_webview)).await;
                                        }
                                    },
                                }
                            }
                            None => break,
                        }
                    }
                }
            }
        });
    });
}

fn main() -> glib::ExitCode {
    if cfg!(debug_assertions) {
        tracing_subscriber::fmt()
            .with_max_level(tracing::Level::INFO) // INFO, WARN, ERROR
            .init();
    }

    // Set GSK_RENDERER to gl, otherwise performance is bombed
    // for many users, if the user wants to modify it, they can
    // simply set it before running the program
    if std::env::var_os("GSK_RENDERER").is_none() {
        unsafe {
            std::env::set_var("GSK_RENDERER", "gl");
        }
    }

    let app = Application::builder()
        .application_id("com.example.wallpaper")
        .build();

    // Tokio is sender
    let (ui_tx, ui_rx) = async_channel::unbounded::<UiCmd>();

    // Glib is sender
    let (bg_tx, bg_rx) = async_channel::unbounded::<UiEvent>();

    info!(target: "main", "Starting Tokio");
    start_tokio(ui_tx, bg_rx);

    // Run application, this function is not fn-once which is why we use
    // async channels over doing acrobatics for tokio's mpsc
    info!(target: "main", "Starting UI");
    app.connect_startup(move |app| build_ui(app, bg_tx.clone(), ui_rx.clone()));
    app.run()
}

struct Instance {
    webview: WebView,
    connector: String,
    wallpaper_path: Option<String>,
}

fn build_ui(
    app: &Application,
    bg_tx: async_channel::Sender<UiEvent>,
    ui_rx: async_channel::Receiver<UiCmd>,
) {
    // Build one window+webview per monitor
    let display = gdk::Display::default().expect("No GDK display");
    let monitors: gio::ListModel = display.monitors();

    let mut instances: Vec<Instance> = Vec::new();

    for i in 0..monitors.n_items() {
        let monitor = monitors
            .item(i)
            .expect("Missing monitor item")
            .downcast::<gdk::Monitor>()
            .expect("Item wasn't a gdk::Monitor");

        let webview = webview::build_ui_for_monitor(app, &monitor, i);
        let connector = monitor.connector().unwrap_or_default().to_string();

        info!(target: "ui", connector = %connector, "Got Connector");
        instances.push(Instance {
            webview,
            connector,
            wallpaper_path: None,
        });
    }

    // Receive loop on GTK thread
    glib::MainContext::default().spawn_local(async move {
        while let Ok(event) = ui_rx.recv().await {
            info!(target: "glib", "Received an event from tokio");
            match event {
                UiCmd::SetWebview(set_webview) => {
                    info!(target: "glib", set_webview = ?set_webview, "Received");
                    if let Some(_monitor) = &set_webview.monitor {
                        // TODO
                        // for inst in &instances {
                        //     // `connector()` exists on many setups; if not, use model/manufacturer/geometry
                        //     if inst.connector == monitor {
                        //         inst.webview.load_uri(&set_webview.url);
                        //         break;
                        //     }
                        // }
                    } else {
                        for inst in &mut instances {
                            let old_path: Option<String> = inst.wallpaper_path.clone();
                            inst.wallpaper_path = set_webview.path.clone();
                            if let Some(path) = old_path {
                                let release_server = ReleaseServer { path };
                                info!(target: "glib", release_server=?release_server, "Sending");
                                let _ = bg_tx
                                    .send(UiEvent::ReleaseServer(release_server))
                                    .await;
                                info!(target: "glib", "Finished Send");
                            }
                            inst.webview.load_uri(&set_webview.url);
                            info!(target: "glib", url = %&set_webview.url, "Loaded")
                        }
                    }
                }
            }
        }

        info!("IPC channel closed");
    });
}
