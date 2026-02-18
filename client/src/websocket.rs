use std::net::TcpStream;

use packets::{ClientPacket, ServerPacket};
use thiserror::Error;
use tungstenite::{Message, WebSocket, connect, stream::MaybeTlsStream};

#[derive(Error, Debug, PartialEq)]
pub enum ConnectionError {
    #[error("failed to connect")]
    FailedToConnect,
}

pub struct Connection {
    url: String,
    socket: Option<WebSocket<MaybeTlsStream<TcpStream>>>,
}

impl Connection {
    pub fn new(url: &str) -> Self {
        Connection {
            socket: None,
            url: format!(
                "ws://{}/game/ws",
                url.trim_start_matches("http://")
                    .trim_start_matches("https://")
            ),
        }
    }

    fn try_get_connection(
        &mut self,
    ) -> Result<&mut WebSocket<MaybeTlsStream<TcpStream>>, ConnectionError> {
        if self.socket.is_none() {
            match connect(self.url.clone()) {
                Ok((socket, _)) => self.socket = Some(socket),
                Err(err) => {
                    eprintln!("Failed to connect: {}", err);
                    return Err(ConnectionError::FailedToConnect);
                }
            }
        }

        self.socket.as_mut().ok_or(ConnectionError::FailedToConnect)
    }

    pub fn send(&mut self, packet: ClientPacket) -> Result<(), ConnectionError> {
        let socket = self.try_get_connection()?;
        socket
            .send(Message::Text(packet.to_string().into()))
            .unwrap();

        Ok(())
    }

    pub fn receive(&mut self) -> Result<ServerPacket, ConnectionError> {
        loop {
            let socket = self.try_get_connection()?;
            let msg = socket.read().unwrap();

            if let Message::Text(text) = msg {
                return ServerPacket::from_string(text.as_ref())
                    .map_err(|_| ConnectionError::FailedToConnect);
            }
        }
    }
}
