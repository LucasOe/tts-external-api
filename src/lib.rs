//! Reference: https://api.tabletopsimulator.com/externaleditorapi/
//!
//! Communication between the editor and TTS occurs via two localhost TCP connections:
//! one where TTS listens for messages and one where ttsst listens for messages.
//! All communication messages are JSON.

mod tcp;

pub use crate::tcp::ExternalEditorApi;
use serde::{
    Deserialize, Serialize, Serializer, __private::ser::FlatMapSerializer, ser::SerializeMap,
};
pub use serde_json::{json, Value};
use std::io::{self};

/////////////////////////////////////////////////////////////////////////////

/// Represents outgoing messages sent to Tabletop Simulator.
#[derive(Debug)]
pub enum Message {
    MessageGetScripts(MessageGetScripts),
    MessageReload(MessageReload),
    MessageCustomMessage(MessageCustomMessage),
    MessageExectute(MessageExectute),
}

// Workaround for: https://github.com/serde-rs/serde/issues/745
// https://stackoverflow.com/questions/65575385/deserialization-of-json-with-serde-by-a-numerical-value-as-type-identifier/65576570#65576570
//
// #[derive(Serialize, Debug)]
// #[serde(tag = "messageID")]
// pub enum Message {
//     #[serde(rename = 0)]
//     MessageGetScripts(MessageGetScripts),
//     #[serde(rename = 1)]
//     MessageReload(MessageReload),
//     #[serde(rename = 2)]
//     MessageCustomMessage(MessageCustomMessage),
//     #[serde(rename = 3)]
//     MessageExectute(MessageExectute),
// }
//
impl Serialize for Message {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut s = serializer.serialize_map(None)?;

        let id_ = &match self {
            Message::MessageGetScripts(_) => 0,
            Message::MessageReload(_) => 1,
            Message::MessageCustomMessage(_) => 2,
            Message::MessageExectute(_) => 3,
        };
        s.serialize_entry("messageID", &id_)?;

        match self {
            Message::MessageGetScripts(t) => t.serialize(FlatMapSerializer(&mut s))?,
            Message::MessageReload(t) => t.serialize(FlatMapSerializer(&mut s))?,
            Message::MessageCustomMessage(t) => t.serialize(FlatMapSerializer(&mut s))?,
            Message::MessageExectute(t) => t.serialize(FlatMapSerializer(&mut s))?,
        }

        s.end()
    }
}

pub struct TryFromMessageError(Message);

/// Get a list containing the states for every object. Returns an [`AnswerReload`] message.
#[derive(Serialize, Debug)]
pub struct MessageGetScripts {}

impl TryFrom<Message> for MessageGetScripts {
    type Error = TryFromMessageError;
    fn try_from(message: Message) -> Result<Self, Self::Error> {
        match message {
            Message::MessageGetScripts(message) => Ok(message),
            other => Err(TryFromMessageError(other)),
        }
    }
}

impl MessageGetScripts {
    pub fn new() -> Self {
        Self {}
    }
}

/// Update the Lua scripts and UI XML for any objects listed in the message,
/// and then reloads the save file, the same way it does when pressing "Save & Play" within the in-game editor.
/// Returns an [`AnswerReload`] message.
///
/// Any objects mentioned have both their Lua script and their UI XML updated.
/// If no value is set for either the "script" or "ui" key then the
/// corresponding Lua script or UI XML is deleted.
#[derive(Serialize, Debug)]
pub struct MessageReload {
    #[serde(rename = "scriptStates")]
    pub script_states: Value,
}

impl TryFrom<Message> for MessageReload {
    type Error = TryFromMessageError;
    fn try_from(message: Message) -> Result<Self, Self::Error> {
        match message {
            Message::MessageReload(message) => Ok(message),
            other => Err(TryFromMessageError(other)),
        }
    }
}

impl MessageReload {
    pub fn new(script_states: Value) -> Self {
        Self { script_states }
    }
}

/// Send a custom message to be forwarded to the `onExternalMessage` event handler
/// in the currently loaded game. The value of customMessage must be an object,
/// and is passed as a parameter to the event handler.
/// If this value is not an object then the event is not triggered.
#[derive(Serialize, Debug)]
pub struct MessageCustomMessage {
    #[serde(rename = "customMessage")]
    pub custom_message: Value,
}

impl TryFrom<Message> for MessageCustomMessage {
    type Error = TryFromMessageError;
    fn try_from(message: Message) -> Result<Self, Self::Error> {
        match message {
            Message::MessageCustomMessage(message) => Ok(message),
            other => Err(TryFromMessageError(other)),
        }
    }
}

impl MessageCustomMessage {
    pub fn new(custom_message: Value) -> Self {
        Self { custom_message }
    }
}

/// Executes a lua script and returns the value in a [`AnswerReturn`] message.
/// Using a guid of "-1" runs the script globally.
#[derive(Serialize, Debug)]
pub struct MessageExectute {
    #[serde(rename = "returnID")]
    pub return_id: u8,
    #[serde(rename = "guid")]
    pub guid: String,
    #[serde(rename = "script")]
    pub script: String,
}

impl TryFrom<Message> for MessageExectute {
    type Error = TryFromMessageError;
    fn try_from(message: Message) -> Result<Self, Self::Error> {
        match message {
            Message::MessageExectute(message) => Ok(message),
            other => Err(TryFromMessageError(other)),
        }
    }
}

impl MessageExectute {
    pub fn new(script: String) -> Self {
        Self {
            return_id: 5,
            guid: String::from("-1"),
            script,
        }
    }
}

/////////////////////////////////////////////////////////////////////////////

/// Represents incoming messages sent by Tabletop Simulator.
#[derive(Debug)]
pub enum Answer {
    AnswerNewObject(AnswerNewObject),
    AnswerReload(AnswerReload),
    AnswerPrint(AnswerPrint),
    AnswerError(AnswerError),
    AnswerCustomMessage(AnswerCustomMessage),
    AnswerReturn(AnswerReturn),
    AnswerGameSaved(AnswerGameSaved),
    AnswerObjectCreated(AnswerObjectCreated),
}

// Workaround for: https://github.com/serde-rs/serde/issues/745
// https://stackoverflow.com/questions/65575385/deserialization-of-json-with-serde-by-a-numerical-value-as-type-identifier/65576570#65576570
//
// #[derive(Deserialize, Debug)]
// #[serde(tag = "messageID")]
// pub enum Answer {
//     #[serde(rename = 0)]
//     AnswerNewObject(AnswerNewObject),
//     #[serde(rename = 1)]
//     AnswerReload(AnswerReload),
//     #[serde(rename = 2)]
//     AnswerPrint(AnswerPrint),
//     #[serde(rename = 3)]
//     AnswerError(AnswerError),
//     #[serde(rename = 4)]
//     AnswerCustomMessage(AnswerCustomMessage),
//     #[serde(rename = 5)]
//     AnswerReturn(AnswerReturn),
//     #[serde(rename = 6)]
//     AnswerGameSaved(AnswerGameSaved),
//     #[serde(rename = 7)]
//     AnswerObjectCreated(AnswerObjectCreated),
// }
//
impl<'de> serde::Deserialize<'de> for Answer {
    fn deserialize<D: serde::Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        let value = Value::deserialize(d)?;

        Ok(
            match value.get("messageID").and_then(Value::as_u64).unwrap() {
                0 => Answer::AnswerNewObject(AnswerNewObject::deserialize(value).unwrap()),
                1 => Answer::AnswerReload(AnswerReload::deserialize(value).unwrap()),
                2 => Answer::AnswerPrint(AnswerPrint::deserialize(value).unwrap()),
                3 => Answer::AnswerError(AnswerError::deserialize(value).unwrap()),
                4 => Answer::AnswerCustomMessage(AnswerCustomMessage::deserialize(value).unwrap()),
                5 => Answer::AnswerReturn(AnswerReturn::deserialize(value).unwrap()),
                6 => Answer::AnswerGameSaved(AnswerGameSaved::deserialize(value).unwrap()),
                7 => Answer::AnswerObjectCreated(AnswerObjectCreated::deserialize(value).unwrap()),
                id_ => panic!("unsupported id {:?}", id_),
            },
        )
    }
}

pub struct TryFromAnswerError(Answer);

/// When clicking on "Scripting Editor" in the right click contextual menu
/// in TTS for an object that doesn't have a Lua Script yet, TTS will send
/// an [`AnswerNewObject`] message containing data for the object.
///
/// # Example
/// ```json
/// {
///     "message_id": 0,
///     "script_states": [
///         {
///             "name": "Chess Pawn",
///             "guid": "db3f06",
///             "script": ""
///         }
///     ]
/// }
/// ```
#[derive(Deserialize, Debug)]
pub struct AnswerNewObject {
    #[serde(rename = "scriptStates")]
    pub script_states: Value,
}

impl TryFrom<Answer> for AnswerNewObject {
    type Error = TryFromAnswerError;
    fn try_from(answer: Answer) -> Result<Self, Self::Error> {
        match answer {
            Answer::AnswerNewObject(message) => Ok(message),
            other => Err(TryFromAnswerError(other)),
        }
    }
}

/// After loading a new game in TTS, TTS will send all the Lua scripts
/// and UI XML from the new game as an [`AnswerReload`].
///
/// TTS sends this message as a response to [`MessageGetScripts`] and [`MessageReload`].
///
/// # Example
/// ```json
/// {
///     "message_id": 1,
///     "script_states": [
///         {
///             "name": "Global",
///             "guid": "-1",
///             "script": "...",
///             "ui": "..."
///         },
///         {
///             "name": "BlackJack Dealer's Deck",
///             "guid": "a0b2d5",
///             "script": "..."
///         },
///     ]
/// }
/// ```
#[derive(Deserialize, Debug)]
pub struct AnswerReload {
    #[serde(rename = "savePath")]
    pub save_path: String,
    #[serde(rename = "scriptStates")]
    pub script_states: Value,
}

impl TryFrom<Answer> for AnswerReload {
    type Error = TryFromAnswerError;
    fn try_from(answer: Answer) -> Result<Self, Self::Error> {
        match answer {
            Answer::AnswerReload(message) => Ok(message),
            other => Err(TryFromAnswerError(other)),
        }
    }
}

impl AnswerReload {
    pub fn script_states(&self) -> Value {
        self.script_states.clone()
    }
}

/// TTS sends all `print()` messages in a [`AnswerPrint`] response.
///
/// # Example
/// ```json
/// {
///     "message_id": 2,
///     "message": "Hit player! White"
/// }
/// ```
#[derive(Deserialize, Debug)]
pub struct AnswerPrint {
    #[serde(rename = "message")]
    pub message: String,
}

impl TryFrom<Answer> for AnswerPrint {
    type Error = TryFromAnswerError;
    fn try_from(answer: Answer) -> Result<Self, Self::Error> {
        match answer {
            Answer::AnswerPrint(message) => Ok(message),
            other => Err(TryFromAnswerError(other)),
        }
    }
}

/// TTS sends all error messages in a [`AnswerError`] response.
///
/// # Example
/// ```json
/// {
///     "message_id": 3,
///     "error": "chunk_0:(36,4-8): unexpected symbol near 'deck'",
///     "guid": "-1",
///     "errorMessagePrefix": "Error in Global Script: "
/// }
/// ```
#[derive(Deserialize, Debug)]
pub struct AnswerError {
    #[serde(rename = "error")]
    pub error: String,
    #[serde(rename = "guid")]
    pub guid: String,
    #[serde(rename = "errorMessagePrefix")]
    pub error_message_prefix: String,
}

impl TryFrom<Answer> for AnswerError {
    type Error = TryFromAnswerError;
    fn try_from(answer: Answer) -> Result<Self, Self::Error> {
        match answer {
            Answer::AnswerError(message) => Ok(message),
            other => Err(TryFromAnswerError(other)),
        }
    }
}

/// Custom Messages are sent by calling [`sendExternalMessage`] with the table of data you wish to send.
///
/// # Example
/// ```json
/// {
///     "message_id": 4,
///     "custom_message": { "foo": "Hello", "bar": "World"}
/// }
/// ```
#[derive(Deserialize, Debug)]
pub struct AnswerCustomMessage {
    #[serde(rename = "customMessage")]
    pub custom_message: Value,
}

impl TryFrom<Answer> for AnswerCustomMessage {
    type Error = TryFromAnswerError;
    fn try_from(answer: Answer) -> Result<Self, Self::Error> {
        match answer {
            Answer::AnswerCustomMessage(message) => Ok(message),
            other => Err(TryFromAnswerError(other)),
        }
    }
}

/// If code executed with a [`MessageExecute`] message returns a value,
/// it will be sent back in a [`AnswerReturn`] message.
/// Tables have to be decoded using `JSON.decode(table)`.
///
/// # Example
/// ```json
/// {
///     "message_id": 5,
///     "return_value": true
/// }
/// ```
#[derive(Deserialize, Debug)]
pub struct AnswerReturn {
    #[serde(rename = "returnID")]
    pub return_id: u8,
    #[serde(rename = "returnValue")]
    pub return_value: Option<Value>,
}

impl TryFrom<Answer> for AnswerReturn {
    type Error = TryFromAnswerError;
    fn try_from(answer: Answer) -> Result<Self, Self::Error> {
        match answer {
            Answer::AnswerReturn(message) => Ok(message),
            other => Err(TryFromAnswerError(other)),
        }
    }
}

impl AnswerReturn {
    /// Returns the return value of the message as a [`Value`]. Valid JSON strings get deserialized if possible.
    /// If deserialization fails JSON strings get returned as a [`Value::String`] instead.
    pub fn return_value(&self) -> Value {
        match self.return_value.clone() {
            None => Value::Null,
            Some(value) => match value {
                Value::String(value) => {
                    serde_json::from_str(&value).unwrap_or(Value::String(value))
                }
                other => other,
            },
        }
    }
}

/// Whenever the player saves the game in TTS, [`AnswerGameSaved`] is sent as a response.
#[derive(Deserialize, Debug)]
pub struct AnswerGameSaved {}

impl TryFrom<Answer> for AnswerGameSaved {
    type Error = TryFromAnswerError;
    fn try_from(answer: Answer) -> Result<Self, Self::Error> {
        match answer {
            Answer::AnswerGameSaved(message) => Ok(message),
            other => Err(TryFromAnswerError(other)),
        }
    }
}

/// Whenever the player saves the game in TTS, [`AnswerObjectCreated`] is sent as a response.
///
/// # Example
/// ```json
/// {
///     "message_id": 7,
///     "guid": "abcdef"
/// }
/// ```
#[derive(Deserialize, Debug)]
pub struct AnswerObjectCreated {
    #[serde(rename = "guid")]
    pub guid: String,
}

impl TryFrom<Answer> for AnswerObjectCreated {
    type Error = TryFromAnswerError;
    fn try_from(answer: Answer) -> Result<Self, Self::Error> {
        match answer {
            Answer::AnswerObjectCreated(message) => Ok(message),
            other => Err(TryFromAnswerError(other)),
        }
    }
}

/////////////////////////////////////////////////////////////////////////////

impl ExternalEditorApi {
    /// Get a list containing the states for every object. Returns an [`AnswerReload`] message on success.
    /// If no connection to the game can be established, an [`io::Error`] gets returned instead.
    pub fn get_scripts(&self) -> io::Result<AnswerReload> {
        self.send(Message::MessageGetScripts(MessageGetScripts::new()))?;
        Ok(self.wait())
    }

    /// Update the Lua scripts and UI XML for any objects listed in the message,
    /// and then reloads the save file, the same way it does when pressing "Save & Play" within the in-game editor.
    /// Returns an [`AnswerReload`] message.
    /// If no connection to the game can be established, an [`io::Error`] gets returned instead.
    ///
    /// Any objects mentioned have both their Lua script and their UI XML updated.
    /// If no value is set for either the "script" or "ui" key then the
    /// corresponding Lua script or UI XML is deleted.
    pub fn reload(&self, script_states: Value) -> io::Result<AnswerReload> {
        self.send(Message::MessageReload(MessageReload::new(script_states)))?;
        Ok(self.wait())
    }

    /// Send a custom message to be forwarded to the `onExternalMessage` event handler
    /// in the currently loaded game. The value of customMessage must be an object,
    /// and is passed as a parameter to the event handler.
    /// If no connection to the game can be established, an [`io::Error`] gets returned.
    ///
    /// If this value is not an object then the event is not triggered.
    pub fn custom_message(&self, message: Value) -> io::Result<()> {
        self.send(Message::MessageCustomMessage(MessageCustomMessage::new(
            message,
        )))?;
        Ok(())
    }

    /// Executes a lua script and returns the value in a [`AnswerReturn`] message.
    /// Using a guid of "-1" runs the script globally.
    /// If no connection to the game can be established, an [`io::Error`] gets returned instead.
    pub fn execute(&self, script: String) -> io::Result<AnswerReturn> {
        self.send(Message::MessageExectute(MessageExectute::new(script)))?;
        Ok(self.wait())
    }
}

/////////////////////////////////////////////////////////////////////////////

#[cfg(test)]
mod tests {
    use crate::{json, AnswerNewObject, ExternalEditorApi, Value};

    #[test]
    fn test_get_scripts() {
        let api = ExternalEditorApi::new();

        let script_states: Value = api.get_scripts().unwrap().script_states();
        println!("{:#?}", script_states);
    }

    #[test]
    fn test_reload() {
        let api = ExternalEditorApi::new();

        let script_states = api.reload(json!([])).unwrap().script_states();
        println!("{:#?}", script_states);
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

        let answer: AnswerNewObject = api.wait();
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
