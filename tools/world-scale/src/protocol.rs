use crate::{AcceptedCommand, CellId, CellManifest, Command, CrateState, PedestrianState};
use serde::{Deserialize, Serialize};

pub const PROTOCOL_VERSION: u32 = 1;

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ClientMessage {
    Hello { version: u32, session_id: u64, cells: Vec<CellId> },
    Command { version: u32, command: Command },
    Subscribe { version: u32, cells: Vec<CellId> },
}

impl ClientMessage {
    pub fn hello(session_id: u64, cells: Vec<i32>) -> Self {
        Self::Hello { version: PROTOCOL_VERSION, session_id, cells: cells.into_iter().map(CellId).collect() }
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ServerMessage {
    Welcome { version: u32, manifest: CellManifest, snapshot: ViewSnapshot },
    Ack { version: u32, outcome: AcceptedCommand },
    Snapshot { version: u32, snapshot: ViewSnapshot },
    Error { version: u32, message: String },
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ViewSnapshot {
    pub tick: u64,
    pub complete: bool,
    pub cells: Vec<CellId>,
    pub crate_state: Option<CrateState>,
    pub pedestrian: Option<PedestrianState>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ProtocolError { UnsupportedVersion(u32) }

#[cfg(test)]
mod tests {
    use super::{ClientMessage, PROTOCOL_VERSION};

    #[test]
    fn protocol_version_is_explicit_in_hello() {
        let json = serde_json::to_string(&ClientMessage::hello(9, vec![0, 1])).unwrap();
        assert!(json.contains(&format!("\"version\":{PROTOCOL_VERSION}")));
    }
}
