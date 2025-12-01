use std::net::TcpStream;

use packets::{ClientPacket, ServerPacket};
use tungstenite::{Message, WebSocket, connect, stream::MaybeTlsStream};

pub struct Connection {
    socket: WebSocket<MaybeTlsStream<TcpStream>>,
}

impl Connection {
    pub fn new(url: &str) -> Self {
        let (socket, _) = connect(format!(
            "ws://{}/game/ws",
            url.trim_start_matches("http://")
                .trim_start_matches("https://")
        ))
        .expect("Can't connect");

        Connection { socket }
    }

    pub fn send(&mut self, packet: ClientPacket) {
        self.socket
            .send(Message::Text(packet.to_string().into()))
            .unwrap();
    }

    pub fn receive(&mut self) -> ServerPacket {
        loop {
            let msg = self.socket.read().unwrap();

            if let Message::Text(text) = msg {
                return ServerPacket::from(text.as_ref()).unwrap();
            }
        }
    }
}
