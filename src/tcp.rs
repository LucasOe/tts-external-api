use crate::{Answer, Message};
use std::io::{self, Read, Write};
use std::net::{TcpListener, TcpStream};

/// A struct representing Tabletop Simulators [External Editor API](https://api.tabletopsimulator.com/externaleditorapi/).
#[derive(Debug)]
pub struct ExternalEditorApi {
    listener: TcpListener,
}

impl ExternalEditorApi {
    /// Creates a new ExternalEditorApi struct and binds the TcpListener to its socket adress.
    pub fn new() -> Self {
        let listener = TcpListener::bind("127.0.0.1:39998").unwrap();
        Self { listener }
    }

    /// Sends a [`Message`] in a TcpStream. If no connection to the game can be established, an [`io::Error`] gets returned.
    pub fn send(&self, message: Message) -> io::Result<()> {
        let mut stream = TcpStream::connect("127.0.0.1:39999")?;
        let json_message = serde_json::to_string(&message).unwrap();
        stream.write_all(json_message.as_bytes()).unwrap();
        stream.flush().unwrap();
        Ok(())
    }

    /// Accepts the next incoming [`Answer`] from the listener.
    /// This function will block the calling thread until a new TCP connection is established and an answer gets recieved.
    pub fn read(&self) -> Answer {
        let (mut stream, _addr) = self.listener.accept().unwrap();
        let mut buffer = String::new();
        stream.read_to_string(&mut buffer).unwrap();
        serde_json::from_str(&buffer).unwrap()
    }

    /// Reads incoming [`Answer`] messages until an answer matches the generic.
    /// This function will block the calling thread until a new TCP connection is established and an answer gets recieved.
    pub fn wait<T: TryFrom<Answer>>(&self) -> T {
        loop {
            if let Ok(answer) = T::try_from(self.read()) {
                return answer;
            }
        }
    }
}
