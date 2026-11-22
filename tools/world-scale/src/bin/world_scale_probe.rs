//! Scriptable native WebSocket observer for recovery verification.

use bevy_world_scale::{
    CellId, ClientMessage, Command, Route, ServerMessage, StableId, ViewSnapshot, PROTOCOL_VERSION,
};
use std::{
    env,
    net::TcpStream,
    process::ExitCode,
    time::{Duration, Instant},
};
use tungstenite::{connect, stream::MaybeTlsStream, Message, WebSocket};

fn main() -> ExitCode {
    match run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("probe failed: {error}");
            ExitCode::FAILURE
        }
    }
}

fn run() -> Result<(), String> {
    let args = env::args().skip(1).collect::<Vec<_>>();
    let session_id = value(&args, "--session")
        .unwrap_or("1")
        .parse::<u64>()
        .map_err(|error| error.to_string())?;
    let url = value(&args, "--server").unwrap_or("ws://127.0.0.1:4200");
    let (mut socket, _) = connect(url).map_err(|error| error.to_string())?;
    if let MaybeTlsStream::Plain(stream) = socket.get_mut() {
        stream
            .set_read_timeout(Some(Duration::from_secs(3)))
            .map_err(|error| error.to_string())?;
    }
    send(&mut socket, &ClientMessage::hello(session_id, vec![0, 1]))?;
    let welcome = receive_until(&mut socket, |message| {
        matches!(message, ServerMessage::Welcome { .. })
    })?;
    let initial = snapshot_of(&welcome).ok_or("welcome had no snapshot")?;

    if args.iter().any(|arg| arg == "--unload-reload") {
        send(
            &mut socket,
            &ClientMessage::Subscribe {
                version: PROTOCOL_VERSION,
                cells: vec![CellId(0)],
            },
        )?;
        let unloaded = receive_complete(&mut socket)?;
        if unloaded.cells != [CellId(0)] {
            return Err("cell unload snapshot did not match subscription".into());
        }
        send(
            &mut socket,
            &ClientMessage::Subscribe {
                version: PROTOCOL_VERSION,
                cells: vec![CellId(0), CellId(1)],
            },
        )?;
        let reloaded = receive_complete(&mut socket)?;
        if reloaded.crate_state.is_none() || reloaded.pedestrian.is_none() {
            return Err("cell reload did not rebuild both stable objects".into());
        }
        println!("cell_unload_reload=ok");
    }

    if args.iter().any(|arg| arg == "--push") {
        let command = ClientMessage::Command {
            version: PROTOCOL_VERSION,
            command: Command::push(session_id, 1, initial.tick + 1, StableId(100)),
        };
        send(&mut socket, &command)?;
        if args.iter().any(|arg| arg == "--duplicate") {
            send(&mut socket, &command)?;
        }
        if args.iter().any(|arg| arg == "--drop-ack") {
            let _ = socket.close(None);
            println!("acknowledgment_dropped_after_send=1");
            return Ok(());
        }
    }

    if args.iter().any(|arg| arg == "--wait-change")
        || args.iter().any(|arg| arg == "--expect-committed")
        || args.iter().any(|arg| arg == "--push")
    {
        let snapshot = receive_committed(&mut socket, initial)?;
        println!(
            "push_count=1 crate_revision={} crate_position={:?} route={:?} pedestrian_progress={}",
            snapshot.crate_state.as_ref().unwrap().revision,
            snapshot.crate_state.as_ref().unwrap().position_mm,
            snapshot.pedestrian.as_ref().unwrap().route,
            snapshot.pedestrian.as_ref().unwrap().route_progress_mm
        );
    }
    Ok(())
}

fn value<'a>(args: &'a [String], flag: &str) -> Option<&'a str> {
    args.iter()
        .position(|arg| arg == flag)
        .and_then(|index| args.get(index + 1))
        .map(String::as_str)
}

fn send(
    socket: &mut WebSocket<MaybeTlsStream<TcpStream>>,
    message: &ClientMessage,
) -> Result<(), String> {
    let text = serde_json::to_string(message).map_err(|error| error.to_string())?;
    socket
        .send(Message::Text(text.into()))
        .map_err(|error| error.to_string())
}

fn receive_until(
    socket: &mut WebSocket<MaybeTlsStream<TcpStream>>,
    predicate: impl Fn(&ServerMessage) -> bool,
) -> Result<ServerMessage, String> {
    let deadline = Instant::now() + Duration::from_secs(5);
    while Instant::now() < deadline {
        match socket.read() {
            Ok(Message::Text(text)) => {
                let message: ServerMessage =
                    serde_json::from_str(&text).map_err(|error| error.to_string())?;
                if predicate(&message) {
                    return Ok(message);
                }
            }
            Ok(_) => {}
            Err(error) => return Err(error.to_string()),
        }
    }
    Err("timed out waiting for server message".into())
}

fn snapshot_of(message: &ServerMessage) -> Option<ViewSnapshot> {
    match message {
        ServerMessage::Welcome { snapshot, .. } | ServerMessage::Snapshot { snapshot, .. } => {
            Some(snapshot.clone())
        }
        _ => None,
    }
}

fn receive_complete(
    socket: &mut WebSocket<MaybeTlsStream<TcpStream>>,
) -> Result<ViewSnapshot, String> {
    let message = receive_until(
        socket,
        |message| matches!(message, ServerMessage::Snapshot { snapshot, .. } if snapshot.complete),
    )?;
    snapshot_of(&message).ok_or_else(|| "missing complete snapshot".into())
}

fn receive_committed(
    socket: &mut WebSocket<MaybeTlsStream<TcpStream>>,
    initial: ViewSnapshot,
) -> Result<ViewSnapshot, String> {
    if committed(&initial) {
        return Ok(initial);
    }
    let message = receive_until(socket, |message| {
        snapshot_of(message).is_some_and(|snapshot| committed(&snapshot))
    })?;
    snapshot_of(&message).ok_or_else(|| "missing committed snapshot".into())
}

fn committed(snapshot: &ViewSnapshot) -> bool {
    snapshot
        .crate_state
        .as_ref()
        .is_some_and(|state| state.position_mm == [1_000, 0] && state.revision == 1)
        && snapshot
            .pedestrian
            .as_ref()
            .is_some_and(|state| state.route == Route::Detour && state.route_progress_mm > 0)
}
