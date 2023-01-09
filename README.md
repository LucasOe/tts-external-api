# tts-external-api

A Rust implementation of the [External Editor API][1] for Tabletop Simulator.

This is intended to make it easier to write development tools and plugins
intead of using the built-in script editor.

[1]: https://api.tabletopsimulator.com/externaleditorapi/

## ExternalEditorApi

This is the client/server representing the _editor_. You can listen for
connections from an active instance of Tabletop Simulator, and send messages
to an active instance.

```rs
use tts_external_api::ExternalEditorApi;

fn main() {
	let api = ExternalEditorApi::new();
	api.execute(String::from("print('Hello World')")).unwrap();
}
```

## Outgoing Messages

You can send four types of outgoing messages:

### [Get Lua Scripts](https://api.tabletopsimulator.com/externaleditorapi/#get-lua-scripts)

```rs
use tts_external_api::{ExternalEditorApi, AnswerReload, Value};

fn get_lua_scripts(api: ExternalEditorApi) {
	let answer_reload: AnswerReload = api.get_scripts().unwrap();
	let script_states: Value = answer_reload.script_states();
	println!("{:#?}", script_states);
}
```

### [Save & Play](https://api.tabletopsimulator.com/externaleditorapi/#save-play)

```rs
use tts_external_api::{json, ExternalEditorApi, AnswerReload};

fn save_and_play(api: ExternalEditorApi) {
	// Objects not mentioned in the script_states are not updated.
	let answer_reload: AnswerReload = api.reload(json!([])).unwrap();
	println!("{:#?}", answer_reload);
}
```

### [Custom Message](https://api.tabletopsimulator.com/externaleditorapi/#custom-message)

```rs
use tts_external_api::{json, ExternalEditorApi, Value};

fn custom_message(api: ExternalEditorApi) {
	let message: Value = json![{"foo": "Foo", "bar": "Bar"}];
	api.custom_message(message).unwrap();
}
```

### [Execute Lua Script](https://api.tabletopsimulator.com/externaleditorapi/#execute-lua-code)

```rs
use tts_external_api::{AnswerReturn, ExternalEditorApi, Value};

fn execute_lua_script(api: ExternalEditorApi) {
    // tables have to be encoded
    let answer_return: AnswerReturn = api.execute(String::from(
        "return JSON.encode({foo = 'Foo', bar = 'Bar'})",
    )).unwrap();
    // `return_value()` will decode strings if possible
    let return_value: Value = answer_return.return_value();
    println!("{:#?}", return_value);
}
```

## Incoming Messages

You can also listen to eight types of incoming messages:

### [Pushing New Object](https://api.tabletopsimulator.com/externaleditorapi/#pushing-new-object)

```rs
use tts_external_api::{AnswerNewObject, ExternalEditorApi};

fn await_new_object(api: ExternalEditorApi) {
    let answer_new_object: AnswerNewObject = api.wait();
    println!("{:#?}", answer_new_object);
}
```

### [Loading a New Game](https://api.tabletopsimulator.com/externaleditorapi/#loading-a-new-game)

```rs
use tts_external_api::{AnswerReload, ExternalEditorApi};

fn await_reload(api: ExternalEditorApi) {
    let answer_reload: AnswerReload = api.wait();
    println!("{:#?}", answer_reload);
}
```

### [Print/Debug Messages](https://api.tabletopsimulator.com/externaleditorapi/#printdebug-messages)

```rs
use tts_external_api::{AnswerPrint, ExternalEditorApi};

fn await_print(api: ExternalEditorApi) {
    let answer_print: AnswerPrint = api.wait();
    println!("{:#?}", answer_print);
}
```

### [Error Messages](https://api.tabletopsimulator.com/externaleditorapi/#error-messages)

```rs
use tts_external_api::{AnswerError, ExternalEditorApi};

fn await_error(api: ExternalEditorApi) {
    let answer_error: AnswerError = api.wait();
    println!("{:#?}", answer_error);
}
```

### [Custom messages](https://api.tabletopsimulator.com/externaleditorapi/#custom-messages)

```rs
use tts_external_api::{AnswerCustomMessage, ExternalEditorApi};

fn await_custom_message(api: ExternalEditorApi) {
    let answer_custom_message: AnswerCustomMessage = api.wait();
    println!("{:#?}", answer_custom_message);
}
```

### [Return messages](https://api.tabletopsimulator.com/externaleditorapi/#return-messages)

```rs
use tts_external_api::{AnswerReturn, ExternalEditorApi};

fn await_return(api: ExternalEditorApi) {
    let answer_return: AnswerReturn = api.wait();
    println!("{:#?}", answer_return);
}
```

### [Game Saved](https://api.tabletopsimulator.com/externaleditorapi/#game-saved)

```rs
use tts_external_api::{AnswerGameSaved, ExternalEditorApi};

fn await_game_saved(api: ExternalEditorApi) {
    let answer_game_saved: AnswerGameSaved = api.wait();
    println!("{:#?}", answer_game_saved);
}
```

### [Object Created](https://api.tabletopsimulator.com/externaleditorapi/#object-created)

```rs
use tts_external_api::{AnswerObjectCreated, ExternalEditorApi};

fn await_object_created(api: ExternalEditorApi) {
    let answer_object_created: AnswerObjectCreated = api.wait();
    println!("{:#?}", answer_object_created);
}
```

## Unspecified Message

Or you can wait for any incoming message:

```rs
use tts_external_api::{Answer, ExternalEditorApi};

fn await_message(api: ExternalEditorApi) {
    let answer: Answer = api.read();
    match answer {
        Answer::AnswerNewObject(_) => println!("pushing new object"),
        Answer::AnswerReload(_) => println!("loading new game"),
        Answer::AnswerPrint(_) => println!("print/debug message"),
        Answer::AnswerError(_) => println!("error message"),
        Answer::AnswerCustomMessage(_) => println!("custom message"),
        Answer::AnswerReturn(_) => println!("return message"),
        Answer::AnswerGameSaved(_) => println!("game saved"),
        Answer::AnswerObjectCreated(_) => println!("object created"),
    }
}
```
