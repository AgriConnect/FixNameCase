use std::io::{Read, Write};
use std::path::PathBuf;
use std::process::{Command, Stdio};
use std::{ops::RangeInclusive, str};

use case_converter::camel_to_snake;
use color_eyre::eyre::Context;
use color_eyre::Result;
use eyre::eyre;
use ignore::{DirEntry, WalkBuilder};
use serde::{Deserialize, Serialize};
use glob_match::glob_match;

#[derive(Serialize, Deserialize, Debug)]
struct Tag {
    name: String,
    path: PathBuf,
    kind: String,
}

const UPPER_LETTERS: RangeInclusive<u8> = b'A'..=b'Z';

#[fncmd::fncmd]
fn main(folder: std::path::PathBuf) -> Result<()> {
    color_eyre::install()?;
    folder.try_exists().wrap_err("Input folder not exist")?;
    let walker = WalkBuilder::new(&folder).filter_entry(|d| {
        let path = d.path();
        if path.is_dir() {
            return true;
        }
        path.to_str().map_or(false, |s| glob_match("*/**/*.{c,cpp,h,hpp}", s))
    }).build();
    let entries: Vec<DirEntry> = walker.filter_map(|entry| entry.ok()).collect();
    let _folder_path = folder.to_str().ok_or(eyre!("Invalid name"))?;
    let mut command = Command::new("ctags");
    command.stdin(Stdio::piped());
    command.stdout(Stdio::piped());
    command.args([
        "--output-format=json",
        "-L",
        "-",
    ]);
    let process = command.spawn()?;
    let file_paths: Vec<String> = entries
        .into_iter()
        .filter_map(|d| d.into_path().into_os_string().into_string().ok())
        .collect();
    let m = file_paths.join("\n");
    process
        .stdin
        .ok_or(eyre!("Failed to grab stdin"))?
        .write_all(m.as_bytes())?;
    let mut out = String::new();
    process
        .stdout
        .ok_or(eyre!("Can not read ctags output"))
        .map(|mut s| s.read_to_string(&mut out))??;
    let lines = out.trim().split("\n");
    let tags: Vec<Tag> = lines
        .filter_map(|l| {
            let tag = serde_json::from_str::<Tag>(l.trim()).ok()?;
            Some(tag)
        })
        .collect();
    let variables: Vec<Tag> = tags
        .into_iter()
        .filter(|t| (t.kind == "variable" || t.kind == "function"))
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
