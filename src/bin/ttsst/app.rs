use anyhow::{bail, Result};
use colorize::AnsiColor;
use inquire::Select;
use regex::Regex;
use serde_json::{json, Value};
use snailquote::unescape;
use std::fs;
use std::path::{Path, PathBuf};
use ttsst::ExternalEditorApi;

struct Tags {
    valid: Vec<Value>,
    invalid: Vec<Value>,
}

/// Attaches the script to an object by adding the script tag and the script,
/// and then reloads the save, the same way it does when pressing "Save & Play".
pub fn attach(api: &mut ExternalEditorApi, path: &PathBuf, guid: Option<String>) -> Result<()> {
    let path = Path::new(path);
    if path.exists() && path.is_file() {
        let file_name = path.file_name().unwrap().to_str().unwrap();
        let guid = match guid {
            Some(guid) => guid,
            None => select_object(api)?,
        };
        let tag = set_tag(api, file_name, &guid)?;
        println!(
            "{} \"{tag}\" as a tag for \"{guid}\"",
            "added:".yellow().bold()
        );
        let file_content = fs::read_to_string(path)?;
        set_script(api, &guid, &file_content, &tag)?;
        api.reload(json!([]));
        println!("{}", "reloaded save!".green().bold());
        set_tag(api, file_name, &guid)?;
        println!("To save the appied tag it is recommended to save the game before reloading.");
    } else {
        bail!("{:?} is not a file", path)
    }
    Ok(())
}

/// Update the lua scripts and reload the save file.
pub fn reload(api: &mut ExternalEditorApi, path: &PathBuf) -> Result<()> {
    // map tags to guids
    let script = format!(
        r#"
            list = {{}}
            for _, obj in pairs(getAllObjects()) do
                if obj.hasAnyTag() then
                    list[obj.guid] = obj.getTags()
                end
            end
            return JSON.encode(list)
        "#,
    );

    let guid_tags = api.execute(script).return_value().expect("No return value");

    // update scripts with setLuaScript(), so objects without a script get updated.
    if let Value::Object(guid_tags) = guid_tags {
        for (guid, tags) in guid_tags {
            if let Value::Array(tags) = tags {
                let valid_tags = get_valid_tags(tags).valid;
                // ensure that the object only has one valid tag
                let valid_tag: Option<String> = match valid_tags.len() {
                    1 => Some(unescape_value(&valid_tags[0])),
                    0 => None,
                    _ => bail!("{} has multiple script tags", guid),
                };

                if let Some(tag) = valid_tag {
                    let file_path = get_file_from_tag(path, &tag, &guid)?;
                    let file_content = fs::read_to_string(file_path)?;
                    set_script(api, &guid, &file_content, &tag)?;
                }
            }
        }
    }

    // get scriptStates
    let save_data = api.get_scripts().script_states();
    let script_states = save_data.as_array().unwrap();

    // add global script to script_list
    let global_path = Path::new(path).join("./Global.ttslua");

    // get global script from file and fallback to existing script the from save data
    let global_script = match fs::read_to_string(global_path) {
        Ok(global_file_content) => global_file_content,
        Err(_) => unescape(&script_states[0].get("script").unwrap().to_string()).unwrap(),
    };
    // get global ui from the save data
    let global_ui = unescape(&script_states[0].get("ui").unwrap().to_string()).unwrap();

    let message = json!([{
        "guid": "-1",
        "script": global_script,
        "ui": global_ui
    }]);
    api.reload(message);
    println!("{}", "reloaded save!".green().bold());

    Ok(())
}

/// Backup current save as file
pub fn backup(api: &mut ExternalEditorApi, path: &PathBuf) -> Result<()> {
    let mut path = PathBuf::from(path);
    path.set_extension("json");
    let save_path = api.get_scripts().save_path;
    fs::copy(&save_path, &path)?;
    println!(
        "{} \"{save_name}\" as \"{path}\"",
        "save:".yellow().bold(),
        save_name = Path::new(&save_path).file_name().unwrap().to_str().unwrap(),
        path = path.to_str().unwrap()
    );
    Ok(())
}

/// Shows the user a list of all objects in the save to select from.
fn select_object(api: &mut ExternalEditorApi) -> Result<String> {
    let objects = get_objects(api)?;
    let selection = Select::new("Select the object to attach the script to:", objects).prompt();
    match selection {
        Ok(selection) => Ok(unescape_value(&selection)),
        Err(_) => bail!("could not select an object to apply the script to"),
    }
}

/// Add the file as a tag. Tags use "scripts/<File>.ttslua" as a naming convention.
// Guid has to be global so objects without scripts can execute code.
fn set_tag(api: &mut ExternalEditorApi, file_name: &str, guid: &str) -> Result<String> {
    // check if guid exists
    let objects = get_objects(api)?;
    if !objects.contains(&json!(&guid)) {
        bail!("\"{guid}\" does not exist")
    }
    // get existing tags for object
    let tag = format!("scripts/{file_name}");
    let script = format!(
        r#"
            return JSON.encode(getObjectFromGUID("{guid}").getTags())
        "#,
    );

    let tags = api.execute(script).return_value().expect("No return value");

    // set new tags for object
    if let Value::Array(tags) = tags {
        let mut tags = get_valid_tags(tags).invalid;
        tags.push(Value::String(String::from(&tag)));
        let script = format!(
            r#"
                tags = JSON.decode("{tags}")
                getObjectFromGUID("{guid}").setTags(tags)
            "#,
            tags = json!(tags).to_string().escape_default(),
        );
        api.execute(script);

        Ok(tag)
    } else {
        bail!("could not set tag for \"{guid}\"")
    }
}

/// Sets the script for the object.
fn set_script(api: &mut ExternalEditorApi, guid: &str, script: &str, tag: &str) -> Result<()> {
    // check if guid exists
    let objects = get_objects(api)?;
    if !objects.contains(&json!(&guid)) {
        bail!("\"{guid}\" does not exist")
    }
    // add lua script for object
    let script = format!(
        r#"
            return JSON.encode(getObjectFromGUID("{guid}").setLuaScript("{}"))
        "#,
        script.escape_default()
    );
    api.execute(script);

    println!("{} {guid} with tag {tag}", "updated:".yellow().bold());
    Ok(())
}

/// Returns a list of all guids
pub fn get_objects(api: &mut ExternalEditorApi) -> Result<Vec<Value>> {
    let script = format!(
        r#"
            list = {{}}
            for _, obj in pairs(getAllObjects()) do
                table.insert(list, obj.guid)
            end
            return JSON.encode(list)
        "#,
    );
    let objects = api.execute(script).return_value().expect("No return value");
    Ok(objects.as_array().unwrap().to_owned())
}

/// Split the tags into valid and non valid tags
// Get the tags that follow the "scripts/<File>.ttslua" naming convention.
fn get_valid_tags(tags: Vec<Value>) -> Tags {
    let exprs = Regex::new(r"^(scripts/)[\d\w]+(\.lua|.ttslua)$").unwrap();
    let (valid, invalid): (Vec<Value>, Vec<Value>) = tags
        .into_iter()
        .partition(|tag| exprs.is_match(&unescape_value(tag)));

    Tags { valid, invalid }
}

/// Gets the corresponding from the path according to the tag. Path has to be a directory.
fn get_file_from_tag(path: &PathBuf, tag: &str, guid: &str) -> Result<String> {
    let path = Path::new(path);
    let file_name = Path::new(&tag).file_name().unwrap();
    if path.exists() && path.is_dir() {
        let file_path = path.join(file_name);
        if file_path.exists() && file_path.is_file() {
            Ok(String::from(file_path.to_string_lossy()))
        } else {
            bail!("file for {:?} with tag {} not found", guid, tag)
        }
    } else {
        bail!("{:?} is not a directory", path)
    }
}

/// Unescapes a Value and returns it as a String.
fn unescape_value(value: &Value) -> String {
    unescape(&value.to_string()).unwrap()
}
