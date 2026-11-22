use crate::{
    AcceptedCommand, CellId, CellManifest, Command, CrateState, PedestrianState, SCHEMA_VERSION,
};
use serde::{Deserialize, Serialize};

pub const PROTOCOL_VERSION: u32 = 1;

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ClientMessage {
    Hello {
        version: u32,
        session_id: u64,
        cells: Vec<CellId>,
    },
    Command {
        version: u32,
        command: Command,
    },
    Subscribe {
        version: u32,
        cells: Vec<CellId>,
    },
}

impl ClientMessage {
    pub fn hello(session_id: u64, cells: Vec<i32>) -> Self {
        Self::Hello {
            version: PROTOCOL_VERSION,
            session_id,
            cells: cells.into_iter().map(CellId).collect(),
        }
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ServerMessage {
    Welcome {
        version: u32,
        manifest: CellManifest,
        snapshot: ViewSnapshot,
    },
    Ack {
        version: u32,
        outcome: AcceptedCommand,
    },
    Snapshot {
        version: u32,
        snapshot: ViewSnapshot,
    },
    Error {
        version: u32,
        message: String,
    },
}

pub fn validate_server_message(message: &ServerMessage) -> Result<(), String> {
    let version = match message {
        ServerMessage::Welcome { version, .. }
        | ServerMessage::Ack { version, .. }
        | ServerMessage::Snapshot { version, .. }
        | ServerMessage::Error { version, .. } => *version,
    };
    if version != PROTOCOL_VERSION {
        return Err(format!(
            "unsupported server protocol version {version} (expected {PROTOCOL_VERSION})"
        ));
    }
    if let ServerMessage::Welcome { manifest, .. } = message {
        if manifest.version != SCHEMA_VERSION {
            return Err(format!(
                "unsupported manifest version {} (expected {SCHEMA_VERSION})",
                manifest.version
            ));
        }
    }
    Ok(())
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
pub enum ProtocolError {
    UnsupportedVersion(u32),
}

#[cfg(test)]
mod tests {
    use super::{validate_server_message, ClientMessage, ServerMessage, PROTOCOL_VERSION};
    use crate::{Fixture, ServerCore, SCHEMA_VERSION};

    #[test]
    fn protocol_version_is_explicit_in_hello() {
        let json = serde_json::to_string(&ClientMessage::hello(9, vec![0, 1])).unwrap();
        assert!(json.contains(&format!("\"version\":{PROTOCOL_VERSION}")));
    }

    #[test]
    fn client_rejects_server_and_manifest_version_mismatch() {
        let path = std::env::temp_dir().join("bevy-world-scale-protocol-version");
        let server = ServerCore::fresh(&path, Fixture::intersection(42)).unwrap();
        let mut manifest = server.fixture().manifest.clone();
        manifest.version = SCHEMA_VERSION + 1;
        let message = ServerMessage::Welcome {
            version: PROTOCOL_VERSION,
            manifest,
            snapshot: server.snapshot(&server.fixture().cells, true),
        };
        assert!(validate_server_message(&message)
            .unwrap_err()
            .contains("manifest version"));

        let message = ServerMessage::Error {
            version: PROTOCOL_VERSION + 1,
            message: "x".into(),
        };
        assert!(validate_server_message(&message)
            .unwrap_err()
            .contains("server protocol version"));
    }
}
