use ttsst::ExternalEditorApi;

fn main() {
    let api = ExternalEditorApi::new();
    let answer = api.execute(String::from("print('Hello World')")).unwrap();
    println!("{:#?}", answer.return_value());
}
