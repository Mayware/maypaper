use gtk4::glib::object::{Cast, ObjectExt};
use gtk4::prelude::{ApplicationExt, ApplicationExtManual, GtkWindowExt};
use gtk4::{Application, ApplicationWindow, glib};

use gtk4::gdk::prelude::DisplayExt;
use gtk4::gio::prelude::ListModelExt;
use gtk4::{gdk, gio};

use gtk4_layer_shell::{Edge, KeyboardMode, Layer, LayerShell};

use maypaper::{Ipc, get_default_socket_path};
use tracing::{error, info};
use webkit6::WebView;
use webkit6::prelude::WebViewExt;

use std::fs;
use std::io::{BufRead, BufReader};
use std::os::unix::net::UnixListener;
use std::thread;

fn main() -> glib::ExitCode {
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

    // Start IPC listening
    let (tx, rx) = async_channel::unbounded::<Ipc>();

    thread::spawn(move || {
        let socket_path = get_default_socket_path();
        let _ = fs::remove_file(&socket_path); // Remove pre-existing socket, if it exists

        let listener = match UnixListener::bind(&socket_path) {
            Ok(listener) => listener,
            Err(e) => {
                error!("IPC: failed to bind socket {:?}: {}", socket_path, e);
                return;
            }
        };

        info!("IPC: listening on {:?}", socket_path);

        for connection in listener.incoming() {
            let stream = match connection {
                Ok(stream) => stream,
                Err(e) => {
                    error!("IPC: accept error: {}", e);
                    continue;
                }
            };

            // Handle each client connection (mypctl usually connects, sends 1 line, exits)
            let tx = tx.clone();
            thread::spawn(move || {
                let reader = BufReader::new(stream);

                for line in reader.lines() {
                    let line = match line {
                        Ok(l) => l,
                        Err(e) => {
                            error!("IPC: read line error: {}", e);
                            break;
                        }
                    };

                    if line.trim().is_empty() {
                        continue;
                    }

                    match serde_json::from_str::<Ipc>(&line) {
                        Ok(msg) => {
                            if let Err(e) = tx.send_blocking(msg) {
                                error!("IPC: channel send failed: {}", e);
                                break;
                            }
                        }
                        Err(e) => {
                            error!("IPC: bad JSON: {} | line: {}", e, line);
                        }
                    }
                }
            });
        }
    });

    // Run application
    app.connect_activate(move |app| build_ui(app, rx.clone()));
    app.run()
}

fn build_ui(app: &Application, rx: async_channel::Receiver<Ipc>) {
    // Build one window+webview per monitor
    let display = gdk::Display::default().expect("No GDK display");
    let monitors: gio::ListModel = display.monitors();

    let mut webviews: Vec<WebView> = Vec::new();

    for i in 0..monitors.n_items() {
        let monitor = monitors
            .item(i)
            .expect("Missing monitor item")
            .downcast::<gdk::Monitor>()
            .expect("Item wasn't a gdk::Monitor");

        let webview = build_ui_for_monitor(app, &monitor, i);
        webviews.push(webview);
    }

    info!("Built {} webview(s)", webviews.len());

    // Receive loop on GTK thread
    glib::MainContext::default().spawn_local(async move {
        while let Ok(msg) = rx.recv().await {
            match msg {
                Ipc::Set { monitor, uri } => {
                    if let Some(monitor) = monitor {
                        if let Some(webview) = webviews.get(monitor) {
                            webview.load_uri(&uri);
                        } else {
                            eprintln!("No webview for monitor index {monitor}");
                        }
                    } else {
                        for webview in &webviews {
                            webview.load_uri(&uri);
                        }
                    }
                }
                Ipc::Reload { monitor } => {
                    if let Some(monitor) = monitor {
                        if let Some(webview) = webviews.get(monitor) {
                            webview.reload();
                        }
                    }
                }
            }
        }

        info!("IPC channel closed");
    });
}

fn build_ui_for_monitor(app: &Application, monitor: &gdk::Monitor, idx: u32) -> WebView {
    let window = ApplicationWindow::new(app);
    window.set_title(Some(&format!("maypaper (info) [{idx}]")));

    window.init_layer_shell();
    window.set_layer(Layer::Background);
    window.set_monitor(Some(monitor));

    window.set_anchor(Edge::Left, true);
    window.set_anchor(Edge::Right, true);
    window.set_anchor(Edge::Top, true);
    window.set_anchor(Edge::Bottom, true);

    window.set_exclusive_zone(-1);
    window.set_keyboard_mode(KeyboardMode::OnDemand);
    window.set_namespace(Some("live-wallpaper"));
    window.set_decorated(false);

    let webview = WebView::new();

    // Muted, unless focused
    webview.set_is_muted(true);
    {
        let webview = webview.clone();
        window.connect_notify_local(Some("is-active"), move |window, _| {
            let active = window.is_active();
            webview.set_is_muted(!active);
        });
    }

    let settings = webview.settings().unwrap();
    settings.set_enable_webgl(true);
    settings.set_enable_webaudio(true);
    settings.set_enable_developer_extras(true);

    webview.set_background_color(&gtk4::gdk::RGBA::new(0.20, 0.20, 0.20, 1.0));
    // webview.load_uri("https://paveldogreat.github.io/WebGL-Fluid-Simulation/"); Cool default for
    // testing

    window.set_child(Some(&webview));
    window.present();

    webview
}
