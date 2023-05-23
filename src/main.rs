use std::process::Command;
use std::{ops::RangeInclusive, str};

use color_eyre::eyre::Context;
use color_eyre::Result;
use eyre::eyre;
use serde::{Deserialize, Serialize};
use case_converter::camel_to_snake;

#[derive(Serialize, Deserialize, Debug)]
struct Tag {
    name: String,
    kind: String,
}

const UPPER_LETTERS: RangeInclusive<u8> = b'A'..=b'Z';

#[fncmd::fncmd]
fn main(folder: std::path::PathBuf) -> Result<()> {
    color_eyre::install()?;
    folder.try_exists().wrap_err("Input folder not exist")?;
    let folder_path = folder.to_str().ok_or(eyre!("Invalid name"))?;
    let mut command = Command::new("ctags");
    command.args([
        "--output-format=json",
        "-R",
        "--exclude=node_modules",
        "--exclude=.pio",
        "--exclude=test",
        "--language=c++",
        folder_path,
    ]);
    let cout = command.output()?;
    let soutput = std::str::from_utf8(&cout.stdout)?;
    let lines = soutput.trim().split("\n");
    let tags: Vec<Tag> = lines
        .filter_map(|l| {
            let tag = serde_json::from_str::<Tag>(l.trim()).ok()?;
            Some(tag)
        })
        .collect();
    let variables: Vec<Tag> = tags
        .into_iter()
        .filter(|t| (t.kind == "variable" || t.kind == "function") && t.name != "File")
        .collect();
    let var_names: Vec<String> = variables.into_iter().map(|t| t.name).collect();
    let non_snake_names: Vec<String> = var_names
        .into_iter()
        .filter(|name| {
            let mut iter = UPPER_LETTERS.clone().into_iter();
            iter.any(|c| name.contains(c as char))
        })
        .collect();
    for name in non_snake_names {
        let new_name = camel_to_snake(name.replace("MQTT", "Mqtt").as_str());
        println!("{} -> {}", name, new_name);
    }
    Ok(())
}
