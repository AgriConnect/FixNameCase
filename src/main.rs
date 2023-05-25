mod helpers;

use std::ffi::OsString;
use std::io::{Read, Write};
use std::path::PathBuf;
use std::process::{Command, Stdio};

use color_eyre::eyre::Context;
use color_eyre::Result;
use console::{style, Emoji};
use convert_case::{Case, Casing};
use eyre::eyre;
use ignore::{DirEntry, WalkBuilder};
use serde::{Deserialize, Serialize};
use unicode_intervals::UnicodeCategory;

use helpers::{filter_c_files, join_filepath_list};

#[derive(Serialize, Deserialize, Debug)]
struct Tag {
    name: String,
    path: PathBuf,
    kind: String,
}

const SYMBOL_KINDS: [&str; 4] = ["variable", "function", "local", "parameter"];
const IRREGULAR_REPLACEMENTS: [(&str, &str); 3] = [
    ("MQTT", "Mqtt"),
    ("selfIP", "selfIp"),
    ("newSN", "newSerialNumber"),
];

/// Extract variable, function names from source folder, by ctags.
fn get_symbols(folder: &PathBuf) -> Result<Vec<String>> {
    let walker = WalkBuilder::new(folder)
        .filter_entry(filter_c_files)
        .build();
    let entries: Vec<DirEntry> = walker.filter_map(|entry| entry.ok()).collect();
    let filepaths: Vec<OsString> = entries
        .clone()
        .into_iter()
        .filter_map(|d| {
            // To exclude directories
            let path = d.into_path();
            if path.is_file() {
                Some(path.into_os_string())
            } else {
                None
            }
        })
        .collect();
    if filepaths.is_empty() {
        return Ok(Vec::new());
    }
    let file_list = join_filepath_list(filepaths);
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
    process
        .stdin
        .ok_or(eyre!("Failed to grab stdin"))?
        .write_all(&file_list)?;
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
        .filter(|t| SYMBOL_KINDS.contains(&t.kind.as_str()))
        .collect();
    let mut var_names: Vec<String> = variables.into_iter().map(|t| t.name).collect();
    var_names.sort_unstable_by_key(|n| n.len());

    // We should replace longest name first
    var_names.reverse();
    Ok(var_names)
}

fn fix_irregulars(name: &str) -> String {
    let mut nn = name.clone().to_string();
    for (s, r) in IRREGULAR_REPLACEMENTS {
        nn = nn.replace(s, r);
    }
    nn
}

fn deduce_new_names(names: Vec<String>) -> Vec<(String, String)> {
    names
        .into_iter()
        .filter_map(|name| {
            let new_name = fix_irregulars(&name).to_case(Case::Snake);
            if new_name == name {
                return None;
            }
            Some((name, new_name))
        })
        .collect()
}

/**
Convert variable and function names to snake_case in C/C++ source code.

Hidden files and files listed in .gitignore are ignored.
*/
#[fncmd::fncmd]
fn main(folder: PathBuf) -> Result<()> {
    color_eyre::install()?;
    folder.try_exists().wrap_err("Input folder not exist")?;
    which::which("ctags").wrap_err("ctags is not found. Please install from https://ctags.io/")?;
    which::which("ambr")
        .wrap_err("ambr is not found. Please install from https://github.com/dalance/amber")?;
    let var_names = get_symbols(&folder)?;
    let upper_letters = unicode_intervals::query().include_categories(UnicodeCategory::UPPERCASE_LETTER).interval_set()?;
    let non_snake_names: Vec<String> = var_names
        .into_iter()
        .filter(|name| {
            name.chars().any(|c| upper_letters.contains(c))
        })
        .collect();
    let replacements = deduce_new_names(non_snake_names);
    if replacements.is_empty() {
        println!("{}", style("Found no names to fix.").yellow());
        return Ok(());
    }
    println!("{}", style("To rename:").blue());
    for (name, new_name) in replacements {
        println!("    {name} -> {new_name}");
        let status = Command::new("ambr")
            .current_dir(&folder)
            .args(["--no-interactive", &name, &new_name])
            .status();
        if status.is_err() {
            continue;
        }
    }
    println!("{} {}", Emoji("🎉", "v"), style("Done!").bright().green());
    Ok(())
}
