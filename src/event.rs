use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Ipc {
    Set { monitor: Option<String>, uri: String },
    Reload { monitor: Option<String> },
}


/*
* BASES
*/

#[derive(Debug, Clone)]
pub struct RequestServer {
    pub path: String,
    pub connector: Option<String>,
}

#[derive(Debug, Clone)]
pub struct AcquireServer {
    pub path: String,
    pub connector: String,
}

#[derive(Debug)]
pub struct ReleaseServer {
    pub(crate) path: String,
}

#[derive(Debug)]
pub struct SetWebview {
    pub(crate) url: String,
    pub(crate) path: Option<String>,
    pub(crate) connector: String,
}

#[derive(Debug)]
pub struct ReloadWebview {
    monitor: Option<usize>,
}

/*
* EVENTS
*/

pub enum TokioEvent {
    IpcEvent(IpcEvent),
    WebEvent(WebEvent),
}

pub enum IpcEvent {
    RequestServer(RequestServer),
    ReloadWebview(ReloadWebview),
}

pub enum WebEvent {
    SetWebview(SetWebview),
}

pub enum UiEvent {
    ReleaseServer(ReleaseServer),
}


/*
* CMDS
*/

pub(crate) enum UiCmd {
    SetWebview(SetWebview),
}

pub enum WebCmd {
    AcquireServer(AcquireServer),
    ReleaseServer(ReleaseServer)
}
