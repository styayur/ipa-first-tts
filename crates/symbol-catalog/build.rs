// SPDX-License-Identifier: MPL-2.0
use std::{env, fs, path::PathBuf};
fn main() {
    println!("cargo:rerun-if-changed=data");
    let mut files: Vec<_> = fs::read_dir("data")
        .unwrap()
        .map(|e| e.unwrap().path())
        .filter(|p| p.extension().is_some_and(|e| e == "json"))
        .collect();
    files.sort();
    let sources = files
        .iter()
        .map(|p| format!("include_str!({:?})", fs::canonicalize(p).unwrap()))
        .collect::<Vec<_>>()
        .join(",");
    fs::write(
        PathBuf::from(env::var_os("OUT_DIR").unwrap()).join("catalog.rs"),
        format!("pub const DATA: &[&str] = &[{sources}];"),
    )
    .unwrap();
}
