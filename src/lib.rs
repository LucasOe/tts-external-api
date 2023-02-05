//! A Rust implementation of the [External Editor API](https://api.tabletopsimulator.com/externaleditorapi/) for Tabletop Simulator.
//!
//! This is intended to make it easier to write development tools and plugins
//! instead of using the built-in script editor.
//!
//! Communication between the editor and TTS occurs via two localhost TCP connections:
//! one where TTS listens for messages and one where the [`ExternalEditorApi`] listens for messages.
//! All communication messages are sent in JSON.
//!
//! # Quick Start
//!
//! All messages are sent and received through the [`ExternalEditorApi`] struct.
//! Creating the struct binds the TcpListener to port 39999. In order for messages to be
//! sent, a game has to be loaded inside Tabletop Simulator.
//! ```
//! use tts_external_api::ExternalEditorApi;
//!
//! fn main() {
//! 	let api = ExternalEditorApi::new();
//!
//!     api.execute(String::from("print('Hello World')"))
//!         .expect("Can't connect. Is a save loaded?");
//! }
//! ```

#![deny(missing_docs)]

mod error;
pub mod messages;
pub mod tcp;

pub use crate::tcp::ExternalEditorApi;
pub use serde_json::{json, Value};

/////////////////////////////////////////////////////////////////////////////

#[cfg(test)]
mod tests {
    use crate::{json, messages, ExternalEditorApi};

    #[test]
    fn test_get_scripts() {
        let api = ExternalEditorApi::new();

        let answer = api.get_scripts().unwrap();
        println!("{:#?}", answer.script_states);
    }

    #[test]
    fn test_reload() {
        let api = ExternalEditorApi::new();

        let answer = api.reload(json!([])).unwrap();
        println!("{:#?}", answer.script_states);
    }

    #[test]
    fn test_custom_message() {
        let api = ExternalEditorApi::new();

        api.custom_message(json![{"foo": "Foo"}]).unwrap();
    }

    #[test]
    fn test_execute() {
        let api = ExternalEditorApi::new();

        let answer = api
            .execute(String::from("return JSON.encode({foo = 'Foo'})"))
            .unwrap();
        println!("{:#?}", answer);
    }

    #[test]
    fn test_new_object() {
        let api = ExternalEditorApi::new();

        let answer: messages::AnswerNewObject = api.wait();
        println!("{:#?}", answer);
    }

    #[test]
    fn test_read() {
        let api = ExternalEditorApi::new();

        loop {
            let answer = api.read();
            println!("{:#?}", answer);
        }
    }
}
