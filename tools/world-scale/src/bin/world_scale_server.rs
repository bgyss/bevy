//! Headless 30 Hz authoritative world-scale server.

use bevy::{app::ScheduleRunnerPlugin, prelude::*};
use bevy_world_scale::{
    CellId, ClientMessage, Command, Fixture, ServerCore, ServerMessage, PROTOCOL_VERSION,
};
use std::{
    collections::BTreeMap,
    env,
    net::{TcpListener, TcpStream},
    path::PathBuf,
    sync::{
        atomic::{AtomicU64, Ordering},
        mpsc::{self, Receiver, Sender},
        Mutex,
    },
    thread,
    time::Duration,
};
use tungstenite::{accept, Error as WebSocketError, Message};

struct Inbound {
    client_id: u64,
    message: ClientMessage,
    outbound: Sender<ServerMessage>,
}

#[derive(Resource)]
struct Inbox(Mutex<Receiver<Inbound>>);

#[derive(Resource)]
struct Authority {
    core: Mutex<ServerCore>,
    clients: Mutex<BTreeMap<u64, (Vec<CellId>, Sender<ServerMessage>)>>,
}

fn main() {
    let mut address = "127.0.0.1:4200".to_owned();
    let mut state_path = PathBuf::from("tools/world-scale/.state");
    let args = env::args().skip(1).collect::<Vec<_>>();
    if let Some(index) = args.iter().position(|arg| arg == "--listen") { address = args[index + 1].clone(); }
    if let Some(index) = args.iter().position(|arg| arg == "--state") { state_path = PathBuf::from(&args[index + 1]); }

    let core = if state_path.join("checkpoint.json").exists() {
        ServerCore::recover(&state_path).expect("recover authoritative state")
    } else {
        ServerCore::fresh(&state_path, Fixture::intersection(42)).expect("create authoritative state")
    };
    let (sender, receiver) = mpsc::channel();
    spawn_listener(address.clone(), sender);
    println!("world-scale server listening on ws://{address} at 30 Hz");

    App::new()
        .add_plugins(MinimalPlugins.set(ScheduleRunnerPlugin::run_loop(Duration::from_secs_f64(1.0 / 30.0))))
        .insert_resource(Time::<Fixed>::from_hz(30.0))
        .insert_resource(Inbox(Mutex::new(receiver)))
        .insert_resource(Authority { core: Mutex::new(core), clients: Mutex::new(BTreeMap::new()) })
        .add_systems(FixedUpdate, authoritative_tick)
        .run();
}

fn spawn_listener(address: String, sender: Sender<Inbound>) {
    thread::spawn(move || {
        let listener = TcpListener::bind(&address).expect("bind WebSocket listener");
        let next_id = AtomicU64::new(1);
        for stream in listener.incoming() {
            match stream {
                Ok(stream) => {
                    let client_id = next_id.fetch_add(1, Ordering::Relaxed);
                    let sender = sender.clone();
                    thread::spawn(move || serve_client(client_id, stream, sender));
                }
                Err(error) => eprintln!("accept failed: {error}"),
            }
        }
    });
}

fn serve_client(client_id: u64, stream: TcpStream, sender: Sender<Inbound>) {
    let mut socket = accept(stream).expect("WebSocket handshake");
    socket.get_mut().set_nonblocking(true).expect("nonblocking socket");
    let (outbound, responses) = mpsc::channel();
    loop {
        match socket.read() {
            Ok(Message::Text(text)) => match serde_json::from_str(&text) {
                Ok(message) => {
                    if sender.send(Inbound { client_id, message, outbound: outbound.clone() }).is_err() { return; }
                }
                Err(error) => {
                    let _ = outbound.send(ServerMessage::Error { version: PROTOCOL_VERSION, message: format!("invalid message: {error}") });
                }
            },
            Ok(Message::Close(_)) => return,
            Ok(_) => {}
            Err(WebSocketError::Io(error)) if error.kind() == std::io::ErrorKind::WouldBlock => {}
            Err(WebSocketError::ConnectionClosed) => return,
            Err(error) => { eprintln!("client {client_id} read failed: {error}"); return; }
        }
        while let Ok(message) = responses.try_recv() {
            let text = serde_json::to_string(&message).expect("serialize server message");
            if socket.send(Message::Text(text.into())).is_err() { return; }
        }
        thread::sleep(Duration::from_millis(2));
    }
}

fn authoritative_tick(inbox: Res<Inbox>, authority: Res<Authority>) {
    let messages = {
        let receiver = inbox.0.lock().expect("inbox lock");
        receiver.try_iter().collect::<Vec<_>>()
    };
    let mut commands: Vec<(u64, Command, Sender<ServerMessage>)> = Vec::new();
    let mut core = authority.core.lock().expect("authority lock");
    let mut clients = authority.clients.lock().expect("clients lock");
    for inbound in messages {
        match inbound.message {
            ClientMessage::Hello { version, cells, .. } => {
                if let Err(error) = ServerCore::validate_protocol(version) {
                    let _ = inbound.outbound.send(ServerMessage::Error { version: PROTOCOL_VERSION, message: format!("{error:?}") });
                    continue;
                }
                clients.insert(inbound.client_id, (cells.clone(), inbound.outbound.clone()));
                let _ = inbound.outbound.send(ServerMessage::Welcome { version: PROTOCOL_VERSION, manifest: core.fixture().manifest.clone(), snapshot: core.snapshot(&cells, true) });
            }
            ClientMessage::Command { version, command } if version == PROTOCOL_VERSION => commands.push((inbound.client_id, command, inbound.outbound)),
            ClientMessage::Subscribe { version, cells } if version == PROTOCOL_VERSION => {
                clients.insert(inbound.client_id, (cells.clone(), inbound.outbound.clone()));
                let _ = inbound.outbound.send(ServerMessage::Snapshot { version: PROTOCOL_VERSION, snapshot: core.snapshot(&cells, true) });
            }
            _ => { let _ = inbound.outbound.send(ServerMessage::Error { version: PROTOCOL_VERSION, message: "protocol version mismatch".into() }); }
        }
    }
    let tick_commands = commands.iter().map(|(_, command, _)| command.clone()).collect::<Vec<_>>();
    if let Err(error) = core.tick(&tick_commands) { eprintln!("durable tick failed: {error}"); return; }
    for (_, command, outbound) in commands {
        let key = format!("{}:{}", command.session_id, command.sequence);
        if let Some(outcome) = core.state().accepted_commands.get(&key) {
            let _ = outbound.send(ServerMessage::Ack { version: PROTOCOL_VERSION, outcome: outcome.clone() });
        }
    }
    for (cells, outbound) in clients.values() {
        let _ = outbound.send(ServerMessage::Snapshot { version: PROTOCOL_VERSION, snapshot: core.snapshot(cells, false) });
    }
}
