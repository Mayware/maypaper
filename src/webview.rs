use gtk4::glib::object::ObjectExt;
use gtk4::prelude::GtkWindowExt;
use gtk4::{Application, ApplicationWindow};

use gtk4::{gdk, gio};

use gtk4_layer_shell::{Edge, KeyboardMode, Layer, LayerShell};

use tracing::{debug, error};
use webkit6::prelude::WebViewExt;
use webkit6::{LoadEvent, NetworkSession, WebView};

pub fn build_ui_for_monitor(app: &Application, monitor: &gdk::Monitor, idx: u32) -> WebView {
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

    // Prevent caching
    let session = NetworkSession::new_ephemeral();
    // let webview = WebView::new();
    let webview = WebView::builder()
        .network_session(&session) // construct-only property
        .build();
    attach_load_handlers(&webview, idx);

    {
        let webview = webview.clone();
        window.connect_notify_local(Some("is-active"), move |window, _| {
            let active = window.is_active();
            webview.set_is_muted(!active);

            let js = format!(
                "globalThis.maypaper?.setFocused({});",
                if active { "true" } else { "false" }
            );

            webview.evaluate_javascript(
                &js,
                None::<&str>,             // world_name
                Some("maypaper://focus"), // source_uri (nice for debugging stack traces)
                None::<&gio::Cancellable>,
                move |result| {
                    if let Err(err) = result {
                        eprintln!("evaluate_javascript failed: {err}");
                    }
                },
            );
        });
    }

    let settings = webview.settings().unwrap();
    settings.set_enable_webgl(true);
    settings.set_enable_webaudio(true);
    settings.set_enable_developer_extras(true);

    webview.set_background_color(&gtk4::gdk::RGBA::new(0.20, 0.20, 0.20, 1.0));
    //webview.load_uri("http://localhost:8000/");

    window.set_child(Some(&webview));
    window.present();

    webview
}

pub fn attach_load_handlers(webview: &WebView, idx: u32) {
    webview.connect_load_failed(move |_wv, event, failing_uri, e| {
        error!(target: "webview", idx = %idx, event = ?event, failing_uri = %failing_uri, error = %e, "Load failed");
        // Show webkit's error page
        false
    });

    // Useful for logging successful finishes too
    webview.connect_load_changed(move |_wv, event| {
        if event == LoadEvent::Finished {
            debug!(target: "webview", idx = %idx, event = ?event, "Load changed");
        }
    });
}
