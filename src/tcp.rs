use crate::{Answer, Message};
use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};

#[derive(Debug)]
pub struct ExternalEditorApi {
    listener: TcpListener,
}

impl ExternalEditorApi {
    pub fn new() -> Self {
        let listener = TcpListener::bind("127.0.0.1:39998").unwrap();
        Self { listener }
    }

    pub fn send(&self, message: Message) {
        let mut stream = TcpStream::connect("127.0.0.1:39999").unwrap();
        let json_message = serde_json::to_string(&message).unwrap();
        stream.write_all(json_message.as_bytes()).unwrap();
        stream.flush().unwrap();
    }

    pub fn read(&self) -> Answer {
        let (mut stream, _addr) = self.listener.accept().unwrap();
        let mut buffer = String::new();
        stream.read_to_string(&mut buffer).unwrap();
        println!("{}", buffer);
        serde_json::from_str(&buffer).unwrap()
    }

    pub fn wait<T: TryFrom<Answer>>(&self) -> T {
        loop {
            if let Ok(answer) = T::try_from(self.read()) {
                return answer;
            }
        }
    }
}
