use std::sync::{Arc, Mutex};

use serde_json;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use web_sys::{ErrorEvent, MessageEvent, WebSocket};

use tarot_lib::game::{events, events_data};
use tarot_lib::game::game::Game;

use crate::websockets::handler_game;
use crate::websockets::handler_ui;
use crate::js_api::log;


macro_rules! console_log {
    ($($t:tt)*) => (log(&format_args!($($t)*).to_string()))
}


struct GameData {
    username: String,
    game: Option<Game>,
    socket: WebSocket,
}


fn on_open(game_data: Arc<Mutex<GameData>>, v: JsValue) {
    console_log!("on_open(): {:?}", v);

    let game_data = game_data.lock().unwrap();

    let event = events::Event::WsConnect(events_data::WsConnectData {
        username: game_data.username.clone(),
    });
    let ret = game_data.socket.send_with_str(&serde_json::to_string(&event).unwrap());

    if let Err(err) = ret {
        console_log!("error sending message: {:?}", err);
    }
}

fn on_close(error: ErrorEvent) {
    console_log!("on_close(): {:?}", error);

    handler_ui::events_append_str("Disconnect from server");
}

fn on_error(error: ErrorEvent) {
    console_log!("on_error(): {:?}", error);

    handler_ui::events_append_str("Connection error. Try refreshing the page.");
}

fn on_message(msg: MessageEvent) {
    let data = msg
        .data()
        .as_string()
        .expect("Can't convert received data to a string");
    let deserialized: events::Event = serde_json::from_str(&data).unwrap();

    console_log!("on_message(): {:?}", deserialized);

    handler_game::on_message(&deserialized);
    handler_ui::on_message(&deserialized);
}


pub fn main(addr: String, username: String) -> Result<(), JsValue> {
    let ws = WebSocket::new(&addr)?;

    let game_data = Arc::new(Mutex::new(GameData {
        username: username.clone(),
        game: None,
        socket: ws.clone(),
    }));

    let c = Closure::wrap(Box::new(move |v| { on_open(Arc::clone(&game_data), v); }) as Box<dyn FnMut(JsValue)>);
    ws.set_onopen(Some(c.as_ref().unchecked_ref()));
    c.forget();

    let c = Closure::wrap(Box::new(move |e| { on_close(e); }) as Box<dyn FnMut(ErrorEvent)>);
    ws.set_onclose(Some(c.as_ref().unchecked_ref()));
    c.forget();

    let c = Closure::wrap(Box::new(move |e| { on_error(e); }) as Box<dyn FnMut(ErrorEvent)>);
    ws.set_onerror(Some(c.as_ref().unchecked_ref()));
    c.forget();

    let c = Closure::wrap(Box::new(move |e| { on_message(e); }) as Box<dyn FnMut(MessageEvent)>);
    ws.set_onmessage(Some(c.as_ref().unchecked_ref()));
    c.forget();

    Ok(())
}
