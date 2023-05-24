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
use which;

#[derive(Serialize, Deserialize, Debug)]
struct Tag {
    name: String,
    path: PathBuf,
    kind: String,
}

const UPPER_LETTERS: RangeInclusive<u8> = b'A'..=b'Z';
const KINDS: [&str; 4] = ["variable", "function", "local", "parameter"];


fn get_symbols(folder: &std::path::PathBuf) -> Result<Vec<String>> {
    let walker = WalkBuilder::new(folder).filter_entry(|d| {
        let path = d.path();
        if path.is_dir() {
            return true;
        }
        path.to_str().map_or(false, |s| glob_match("*/**/*.{c,cpp,h,hpp,ino}", s))
    }).build();
    let entries: Vec<DirEntry> = walker.filter_map(|entry| entry.ok()).collect();
    let _folder_path = folder.to_str().ok_or(eyre!("Invalid name"))?;
    let mut command = Command::new("ctags");
    command.stdin(Stdio::piped());
    command.stdout(Stdio::piped());
    command.args([
        "--output-format=json",
        // Also extract local variables and function parameters
        "--kinds-c=+lz",
        "-L",
        "-",
    ]);
    let process = command.spawn()?;
    let file_paths: Vec<String> = entries
        .into_iter()
        .filter_map(|d| d.into_path().into_os_string().into_string().ok())
        .collect();
    process
        .stdin
        .ok_or(eyre!("Failed to grab stdin"))?
        .write_all(file_paths.join("\n").as_bytes())?;
    let mut out = String::new();
    process
        .stdout
        .ok_or(eyre!("Can not read ctags output"))
        .map(|mut s| s.read_to_string(&mut out))??;
    let lines = out.trim().lines();
    let tags: Vec<Tag> = lines
        .filter_map(|l| serde_json::from_str::<Tag>(l.trim()).ok())
        .collect();
    let variables: Vec<Tag> = tags
        .into_iter()
        .filter(|t| KINDS.contains(&t.kind.as_str()))
        .collect();
    let mut var_names: Vec<String> = variables.into_iter().map(|t| t.name).collect();
    var_names.sort_unstable_by_key(|n| n.len());
    var_names.reverse();
    Ok(var_names)
}


fn replace_irregulars(name: String) -> String {
    name.replace("MQTT", "Mqtt").replace("selfIP", "selfIp").replace("newSN", "newSerialNumber")
}

#[fncmd::fncmd]
fn main(folder: std::path::PathBuf) -> Result<()> {
    color_eyre::install()?;
    folder.try_exists().wrap_err("Input folder not exist")?;
    which::which("ctags").wrap_err("ctags is not found. Please install from https://ctags.io/")?;
    which::which("ambr").wrap_err("ambr is not found. Please install from https://github.com/dalance/amber")?;
    let var_names = get_symbols(&folder)?;
    let non_snake_names: Vec<String> = var_names
        .into_iter()
        .filter(|name| {
            let mut iter = UPPER_LETTERS.clone().into_iter();
            iter.any(|c| name.contains(c as char))
        })
        .collect();
    for name in non_snake_names {
        let new_name = camel_to_snake(replace_irregulars(name.clone()).as_str());
        println!("To rename: {} -> {}", name, new_name);
        let status = Command::new("ambr").current_dir(&folder).args(["--no-interactive", &name, &new_name]).status();
        if status.is_err() {
            continue;
        }
    }
    Ok(())
}
