//! Incoming and Outgoing messages

use crate::{error::Error, tcp::ExternalEditorApi, Value};
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use serde::{__private::ser::FlatMapSerializer, ser::SerializeMap};
use std::io::{self};

/////////////////////////////////////////////////////////////////////////////

/// Represents outgoing messages sent to Tabletop Simulator
#[derive(Debug)]
pub enum Message {
    /// Represents [Get Lua Scripts](https://api.tabletopsimulator.com/externaleditorapi/#get-lua-scripts)
    MessageGetScripts(MessageGetScripts),
    /// Represents [Save & Play](https://api.tabletopsimulator.com/externaleditorapi/#get-lua-scripts)
    MessageReload(MessageReload),
    /// Represents [Custom Message](https://api.tabletopsimulator.com/externaleditorapi/#custom-message)
    MessageCustomMessage(MessageCustomMessage),
    /// Represents [Execute Lua Code](https://api.tabletopsimulator.com/externaleditorapi/#execute-lua-code)
    MessageExecute(MessageExecute),
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
//     MessageExecute(MessageExecute),
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
            Message::MessageExecute(_) => 3,
        };
        s.serialize_entry("messageID", &id_)?;

        match self {
            Message::MessageGetScripts(t) => t.serialize(FlatMapSerializer(&mut s))?,
            Message::MessageReload(t) => t.serialize(FlatMapSerializer(&mut s))?,
            Message::MessageCustomMessage(t) => t.serialize(FlatMapSerializer(&mut s))?,
            Message::MessageExecute(t) => t.serialize(FlatMapSerializer(&mut s))?,
        }

        s.end()
    }
}

/// Get a list containing the states for every object. Returns an [`AnswerReload`] message.
#[derive(Serialize, Debug)]
pub struct MessageGetScripts {}

impl TryFrom<Message> for MessageGetScripts {
    type Error = Error;
    fn try_from(message: Message) -> Result<Self, Self::Error> {
        match message {
            Message::MessageGetScripts(message) => Ok(message),
            other => Err(Error::MessageError(other)),
        }
    }
}

impl MessageGetScripts {
    /// Constructs a new Get Lua Scripts Message
    pub fn new() -> Self {
        Self {}
    }

    /// Returns self as [`Message::MessageGetScripts`]
    pub fn as_message(self) -> Message {
        Message::MessageGetScripts(self)
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
    /// Contains a list objects and their state
    #[serde(rename = "scriptStates")]
    pub script_states: Value,
}

impl TryFrom<Message> for MessageReload {
    type Error = Error;
    fn try_from(message: Message) -> Result<Self, Self::Error> {
        match message {
            Message::MessageReload(message) => Ok(message),
            other => Err(Error::MessageError(other)),
        }
    }
}

impl MessageReload {
    /// Constructs a new Save & Play Message
    pub fn new(script_states: Value) -> Self {
        Self { script_states }
    }

    /// Returns self as [`Message::MessageReload`]
    pub fn as_message(self) -> Message {
        Message::MessageReload(self)
    }
}

/// Send a custom message to be forwarded to the `onExternalMessage` event handler
/// in the currently loaded game. The value of customMessage must be an object,
/// and is passed as a parameter to the event handler.
/// If this value is not an object then the event is not triggered.
#[derive(Serialize, Debug)]
pub struct MessageCustomMessage {
    /// Custom message that gets forwarded
    #[serde(rename = "customMessage")]
    pub custom_message: Value,
}

impl TryFrom<Message> for MessageCustomMessage {
    type Error = Error;
    fn try_from(message: Message) -> Result<Self, Self::Error> {
        match message {
            Message::MessageCustomMessage(message) => Ok(message),
            other => Err(Error::MessageError(other)),
        }
    }
}

impl MessageCustomMessage {
    /// Constructs a new Custom Message
    pub fn new(custom_message: Value) -> Self {
        Self { custom_message }
    }

    /// Returns self as [`Message::MessageCustomMessage`]
    pub fn as_message(self) -> Message {
        Message::MessageCustomMessage(self)
    }
}

/// Executes a lua script and returns the value in a [`AnswerReturn`] message.
/// Using a guid of "-1" runs the script globally.
#[derive(Serialize, Debug)]
pub struct MessageExecute {
    /// Return Id of the execute message
    #[serde(rename = "returnID")]
    pub return_id: u64,
    /// The guid the message gets executed on
    #[serde(rename = "guid")]
    pub guid: String,
    /// The script that gets executed
    #[serde(rename = "script")]
    pub script: String,
}

impl TryFrom<Message> for MessageExecute {
    type Error = Error;
    fn try_from(message: Message) -> Result<Self, Self::Error> {
        match message {
            Message::MessageExecute(message) => Ok(message),
            other => Err(Error::MessageError(other)),
        }
    }
}

impl MessageExecute {
    /// Constructs a new Execute Lua Code Message that executes code globally
    pub fn new(script: String) -> Self {
        Self {
            return_id: 5,
            guid: String::from("-1"),
            script,
        }
    }

    /// Constructs a new Execute Lua Code Message that executes code on an object
    pub fn new_object(script: String, guid: String) -> Self {
        Self {
            return_id: 5,
            guid,
            script,
        }
    }

    /// Returns self as [`Message::MessageExecute`]
    pub fn as_message(self) -> Message {
        Message::MessageExecute(self)
    }
}

/////////////////////////////////////////////////////////////////////////////

/// Represents incoming messages sent by Tabletop Simulator.
#[derive(Debug)]
pub enum Answer {
    /// Represents [Pushing New Object](https://api.tabletopsimulator.com/externaleditorapi/#pushing-new-object)
    AnswerNewObject(AnswerNewObject),
    /// Represents [Loading a new Game](https://api.tabletopsimulator.com/externaleditorapi/#loading-a-new-game)
    AnswerReload(AnswerReload),
    /// Represents [Print/Debug Messages](https://api.tabletopsimulator.com/externaleditorapi/#printdebug-messages)
    AnswerPrint(AnswerPrint),
    /// Represents [Error Messages](https://api.tabletopsimulator.com/externaleditorapi/#error-messages)
    AnswerError(AnswerError),
    /// Represents [Custom Messages](https://api.tabletopsimulator.com/externaleditorapi/#custom-messages)
    AnswerCustomMessage(AnswerCustomMessage),
    /// Represents [Return Messages](https://api.tabletopsimulator.com/externaleditorapi/#return-messages)
    AnswerReturn(AnswerReturn),
    /// Represents [Game Saved](https://api.tabletopsimulator.com/externaleditorapi/#game-saved)
    AnswerGameSaved(AnswerGameSaved),
    /// Represents [Object Created](https://api.tabletopsimulator.com/externaleditorapi/#object-created)
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
    /// Contains the state of the object
    #[serde(rename = "scriptStates")]
    pub script_states: Value,
}

impl TryFrom<Answer> for AnswerNewObject {
    type Error = Error;
    fn try_from(answer: Answer) -> Result<Self, Self::Error> {
        match answer {
            Answer::AnswerNewObject(message) => Ok(message),
            other => Err(Error::AnswerError(other)),
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
    /// Path to the save file of the current save
    #[serde(rename = "savePath")]
    pub save_path: String,
    /// Contains a list objects and their state
    #[serde(rename = "scriptStates")]
    pub script_states: Value,
}

impl TryFrom<Answer> for AnswerReload {
    type Error = Error;
    fn try_from(answer: Answer) -> Result<Self, Self::Error> {
        match answer {
            Answer::AnswerReload(message) => Ok(message),
            other => Err(Error::AnswerError(other)),
        }
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
    /// Message that got printed
    #[serde(rename = "message")]
    pub message: String,
}

impl TryFrom<Answer> for AnswerPrint {
    type Error = Error;
    fn try_from(answer: Answer) -> Result<Self, Self::Error> {
        match answer {
            Answer::AnswerPrint(message) => Ok(message),
            other => Err(Error::AnswerError(other)),
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
    /// Description of the error
    #[serde(rename = "error")]
    pub error: String,
    /// Guid of the object that has the error
    #[serde(rename = "guid")]
    pub guid: String,
    /// Description of the error
    #[serde(rename = "errorMessagePrefix")]
    pub error_message_prefix: String,
}

impl TryFrom<Answer> for AnswerError {
    type Error = Error;
    fn try_from(answer: Answer) -> Result<Self, Self::Error> {
        match answer {
            Answer::AnswerError(message) => Ok(message),
            other => Err(Error::AnswerError(other)),
        }
    }
}

/// Custom Messages are sent by calling `sendExternalMessage` with the table of data you wish to send.
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
    /// Content of the custom message
    #[serde(rename = "customMessage")]
    pub custom_message: Value,
}

impl TryFrom<Answer> for AnswerCustomMessage {
    type Error = Error;
    fn try_from(answer: Answer) -> Result<Self, Self::Error> {
        match answer {
            Answer::AnswerCustomMessage(message) => Ok(message),
            other => Err(Error::AnswerError(other)),
        }
    }
}

/// If code executed with a [`MessageExecute`] message returns a value,
/// it will be sent back in a [`AnswerReturn`] message.
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
    /// Return Id of message that got executed
    #[serde(rename = "returnID")]
    pub return_id: u64,
    #[serde(
        rename = "returnValue",
        deserialize_with = "deserialize_json_string",
        default
    )]
    /// The Value that got returned
    pub return_value: Value,
}

impl TryFrom<Answer> for AnswerReturn {
    type Error = Error;
    fn try_from(answer: Answer) -> Result<Self, Self::Error> {
        match answer {
            Answer::AnswerReturn(message) => Ok(message),
            other => Err(Error::AnswerError(other)),
        }
    }
}

/// Returns the return value of the message as a [`Value`]. Valid JSON strings get deserialized if possible.
/// If deserialization fails JSON strings get returned as a [`Value::String`] instead.
fn deserialize_json_string<'de, D>(deserializer: D) -> Result<Value, D::Error>
where
    D: Deserializer<'de>,
{
    match Option::deserialize(deserializer)? {
        Some(val) => match val {
            Value::String(val) => Ok(serde_json::from_str(&val).unwrap_or(Value::String(val))),
            other => Ok(other),
        },
        None => Ok(Value::Null),
    }
}

/// Whenever the player saves the game in TTS, [`AnswerGameSaved`] is sent as a response.
#[derive(Deserialize, Debug)]
pub struct AnswerGameSaved {}

impl TryFrom<Answer> for AnswerGameSaved {
    type Error = Error;
    fn try_from(answer: Answer) -> Result<Self, Self::Error> {
        match answer {
            Answer::AnswerGameSaved(message) => Ok(message),
            other => Err(Error::AnswerError(other)),
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
    /// Guid of the object that got created
    #[serde(rename = "guid")]
    pub guid: String,
}

impl TryFrom<Answer> for AnswerObjectCreated {
    type Error = Error;
    fn try_from(answer: Answer) -> Result<Self, Self::Error> {
        match answer {
            Answer::AnswerObjectCreated(message) => Ok(message),
            other => Err(Error::AnswerError(other)),
        }
    }
}

/////////////////////////////////////////////////////////////////////////////

impl ExternalEditorApi {
    /// Get a list containing the states for every object. Returns an [`AnswerReload`] message on success.
    /// If no connection to the game can be established, an [`io::Error`] gets returned instead.
    pub fn get_scripts(&self) -> io::Result<AnswerReload> {
        self.send(MessageGetScripts::new().as_message())?;
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
        self.send(MessageReload::new(script_states).as_message())?;
        Ok(self.wait())
    }

    /// Send a custom message to be forwarded to the `onExternalMessage` event handler
    /// in the currently loaded game. The value of customMessage must be an object,
    /// and is passed as a parameter to the event handler.
    /// If no connection to the game can be established, an [`io::Error`] gets returned.
    ///
    /// If this value is not an object then the event is not triggered.
    pub fn custom_message(&self, message: Value) -> io::Result<()> {
        self.send(MessageCustomMessage::new(message).as_message())?;
        Ok(())
    }

    /// Executes a lua script globally and returns the value in a [`AnswerReturn`] message.
    /// If no connection to the game can be established, an [`io::Error`] gets returned instead.
    pub fn execute(&self, script: String) -> io::Result<AnswerReturn> {
        self.send(MessageExecute::new(script).as_message())?;
        Ok(self.wait())
    }

    /// Executes a lua script on an object and returns the value in a [`AnswerReturn`] message.
    /// If no connection to the game can be established, an [`io::Error`] gets returned instead.
    ///
    /// To execute Lua code for an object in the game that object must have an associated script in TTS.
    /// Otherwise the TTS scripting engine will fail with an error "function \<executeScript>:
    /// Object reference not set to an instance of an object".
    /// Once the in-game editor shows a script associated with an object
    /// then TTS will be able to execute Lua code sent via JSON message for that object.
    pub fn execute_on_object(&self, script: String, guid: String) -> io::Result<AnswerReturn> {
        self.send(MessageExecute::new_object(script, guid).as_message())?;
        Ok(self.wait())
    }
}
