use std::net::{SocketAddr, UdpSocket};

use serde::{Deserialize, Serialize};

use crate::COMMANDS;
use crate::game::{Field, Game, MODE};

#[derive(Serialize, Deserialize)]
pub struct UdpFrame {
    pub(crate) snake1: Vec<Field>,
    pub(crate) snake2: Vec<Field>,
    pub(crate) food: Field,
}

pub fn init_network (game: &Game, port: &str, address: &str) -> UdpSocket {
    let socket: UdpSocket;

    match game.get_mode() {
        MODE::Server => {
            socket = UdpSocket::bind(format!("0.0.0.0:{}", port)).expect("couldn't bind to address");
        }
        MODE::Client => {
            socket = UdpSocket::bind("0.0.0.0:10000").expect("couldn't bind to address");
            let target_server_address: SocketAddr = address.parse().expect("Unable to parse socket address");
            socket.connect(target_server_address).expect("connect function failed");

            // send any msg to server to connect
            send_connect(&socket);
        }
        MODE::Single => {
            socket = UdpSocket::bind("127.0.0.1:10000").expect("couldn't bind to address");
        }
    }
    socket.set_nonblocking(true).unwrap();
    socket
}

pub fn send_connect(socket: &UdpSocket) {
    let serialized = bincode::serialize(&COMMANDS::Connect).unwrap();
    socket.send(&serialized).expect("couldn't send message");
}

pub fn send_endgame(socket: &UdpSocket) {
    let serialized = bincode::serialize(&COMMANDS::Endgame).unwrap();
    socket.send(&serialized).expect("couldn't send message");
}
