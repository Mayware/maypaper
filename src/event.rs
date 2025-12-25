use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Ipc {
    SetPath { monitor: Option<String>, path: String },
    SetUrl { monitor: Option<String>, url: String },
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
    pub path: String,
}

#[derive(Debug)]
pub struct RequestWebview {
    pub url: String,
    pub connector: Option<String>, 
}

#[derive(Debug)]
pub struct SetWebview {
    pub url: String,
    pub path: Option<String>,
    pub connector: String,
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
    RequestWebview(RequestWebview)
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

pub enum UiCmd {
    SetWebview(SetWebview),
}

pub enum WebCmd {
    AcquireServer(AcquireServer),
    ReleaseServer(ReleaseServer)
}
