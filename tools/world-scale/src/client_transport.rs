use crate::{ClientMessage, ClientView, Command, ServerMessage, StableId, PROTOCOL_VERSION};
use bevy::prelude::*;

fn apply_message(view: &mut ClientView, message: ServerMessage) {
    match message {
        ServerMessage::Welcome { snapshot, .. } | ServerMessage::Snapshot { snapshot, .. } => {
            info!("applied server tick {} complete={}", snapshot.tick, snapshot.complete);
            view.apply(snapshot);
        }
        ServerMessage::Ack { outcome, .. } => info!("ack session={} sequence={} accepted_tick={} revision={}", outcome.session_id, outcome.sequence, outcome.accepted_tick, outcome.resulting_revision),
        ServerMessage::Error { message, .. } => error!("server error: {message}"),
    }
}

#[cfg(not(target_arch = "wasm32"))]
mod native {
    use super::*;
    use std::{sync::{mpsc::{self, Receiver, Sender}, Mutex}, thread, time::Duration};
    use tungstenite::{connect, Error as WebSocketError, Message};

    #[derive(Resource)]
    pub struct NativeTransport { outgoing: Sender<ClientMessage>, incoming: Mutex<Receiver<ServerMessage>> }

    pub fn connect_native(mut commands: Commands) {
        let url = std::env::args().skip_while(|arg| arg != "--server").nth(1).unwrap_or_else(|| "ws://127.0.0.1:4200".into());
        let (outgoing, commands_rx) = mpsc::channel::<ClientMessage>();
        let (messages_tx, incoming) = mpsc::channel::<ServerMessage>();
        thread::spawn(move || {
            let (mut socket, _) = connect(&url).expect("connect world-scale server");
            if let tungstenite::stream::MaybeTlsStream::Plain(stream) = socket.get_mut() {
                stream.set_nonblocking(true).expect("nonblocking client");
            }
            socket.send(Message::Text(serde_json::to_string(&ClientMessage::hello(1, vec![0, 1])).unwrap().into())).unwrap();
            loop {
                while let Ok(message) = commands_rx.try_recv() {
                    if socket.send(Message::Text(serde_json::to_string(&message).unwrap().into())).is_err() { return; }
                }
                match socket.read() {
                    Ok(Message::Text(text)) => if let Ok(message) = serde_json::from_str(&text) { let _ = messages_tx.send(message); },
                    Ok(Message::Close(_)) => return,
                    Ok(_) => {}
                    Err(WebSocketError::Io(error)) if error.kind() == std::io::ErrorKind::WouldBlock => {}
                    Err(WebSocketError::ConnectionClosed) => return,
                    Err(error) => { eprintln!("client WebSocket failed: {error}"); return; }
                }
                thread::sleep(Duration::from_millis(2));
            }
        });
        commands.insert_resource(NativeTransport { outgoing, incoming: Mutex::new(incoming) });
    }

    pub fn receive_native(transport: Res<NativeTransport>, mut view: ResMut<ClientView>) {
        for message in transport.incoming.lock().unwrap().try_iter() { apply_message(&mut view, message); }
    }

    pub fn native_controls(keys: Res<ButtonInput<KeyCode>>, transport: Res<NativeTransport>, view: Res<ClientView>) {
        let message = if keys.just_pressed(KeyCode::Space) {
            Some(ClientMessage::Command { version: PROTOCOL_VERSION, command: Command::push(1, 1, view.last_tick + 1, StableId(100)) })
        } else if keys.just_pressed(KeyCode::KeyU) {
            Some(ClientMessage::Subscribe { version: PROTOCOL_VERSION, cells: vec![crate::CellId(0)] })
        } else if keys.just_pressed(KeyCode::KeyR) {
            Some(ClientMessage::Subscribe { version: PROTOCOL_VERSION, cells: vec![crate::CellId(0), crate::CellId(1)] })
        } else { None };
        if let Some(message) = message { let _ = transport.outgoing.send(message); }
    }
}

#[cfg(not(target_arch = "wasm32"))]
pub use native::*;

#[cfg(target_arch = "wasm32")]
mod browser {
    use super::*;
    use std::{cell::RefCell, rc::Rc};
    use wasm_bindgen::{closure::Closure, JsCast};
    use web_sys::{MessageEvent, WebSocket};

    thread_local! {
        static SOCKET: RefCell<Option<WebSocket>> = const { RefCell::new(None) };
        static MESSAGES: Rc<RefCell<Vec<ServerMessage>>> = Rc::new(RefCell::new(Vec::new()));
    }

    pub fn connect_browser() {
        let socket = WebSocket::new("ws://127.0.0.1:4200").expect("create browser WebSocket");
        let onopen_socket = socket.clone();
        let onopen = Closure::<dyn FnMut()>::new(move || {
            let hello = serde_json::to_string(&ClientMessage::hello(2, vec![0, 1])).unwrap();
            onopen_socket.send_with_str(&hello).unwrap();
        });
        socket.set_onopen(Some(onopen.as_ref().unchecked_ref()));
        onopen.forget();
        let onmessage = Closure::<dyn FnMut(MessageEvent)>::new(move |event: MessageEvent| {
            if let Some(text) = event.data().as_string() {
                if let Ok(message) = serde_json::from_str(&text) { MESSAGES.with(|messages| messages.borrow_mut().push(message)); }
            }
        });
        socket.set_onmessage(Some(onmessage.as_ref().unchecked_ref()));
        onmessage.forget();
        SOCKET.with(|stored| *stored.borrow_mut() = Some(socket));
    }

    pub fn receive_browser(mut view: ResMut<ClientView>) {
        MESSAGES.with(|messages| for message in messages.borrow_mut().drain(..) {
            let tick = match &message {
                ServerMessage::Welcome { snapshot, .. } | ServerMessage::Snapshot { snapshot, .. } => Some(snapshot.tick),
                _ => None,
            };
            apply_message(&mut view, message);
            if let Some(tick) = tick { set_status(&format!("connected — authoritative tick {tick}")); }
        });
    }

    pub fn browser_controls(keys: Res<ButtonInput<KeyCode>>, view: Res<ClientView>) {
        let message = if keys.just_pressed(KeyCode::Space) {
            Some(ClientMessage::Command { version: PROTOCOL_VERSION, command: Command::push(2, 1, view.last_tick + 1, StableId(100)) })
        } else if keys.just_pressed(KeyCode::KeyU) {
            Some(ClientMessage::Subscribe { version: PROTOCOL_VERSION, cells: vec![crate::CellId(0)] })
        } else if keys.just_pressed(KeyCode::KeyR) {
            Some(ClientMessage::Subscribe { version: PROTOCOL_VERSION, cells: vec![crate::CellId(0), crate::CellId(1)] })
        } else { None };
        if let Some(message) = message {
            let text = serde_json::to_string(&message).unwrap();
            SOCKET.with(|socket| if let Some(socket) = socket.borrow().as_ref() { let _ = socket.send_with_str(&text); });
        }
    }

    fn set_status(text: &str) {
        if let Some(element) = web_sys::window().and_then(|window| window.document()).and_then(|document| document.get_element_by_id("status")) {
            element.set_text_content(Some(text));
        }
    }
}

#[cfg(target_arch = "wasm32")]
pub use browser::*;
