use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Ipc {
    Set { monitor: Option<usize>, uri: String },
    Reload { monitor: Option<usize> },
}


/*
* BASES
*/

#[derive(Debug)]
pub struct AcquireServer {
    pub path: String,
    pub monitor: Option<usize>,
}

#[derive(Debug)]
pub struct ReleaseServer {
    pub(crate) path: String,
}

#[derive(Debug)]
pub struct SetWebview {
    pub(crate) url: String,
    pub(crate) path: Option<String>,
    pub(crate) monitor: Option<usize>,
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
    AcquireServer(AcquireServer),
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
