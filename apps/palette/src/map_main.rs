// SPDX-License-Identifier: GPL-3.0-or-later
#![forbid(unsafe_code)]
fn main() {
    let args: Vec<_> = std::env::args().skip(1).collect();
    match language_mapping::cli::run(&args) {
        Ok(value) => {
            println!(
                "{}",
                serde_json::to_string_pretty(&value).expect("JSON value")
            );
            if matches!(
                value.get("status").and_then(|s| s.as_str()),
                Some("unsupported" | "needs_selection")
            ) {
                std::process::exit(2);
            }
        }
        Err(error) => {
            eprintln!("{error}");
            println!(
                "{}",
                serde_json::json!({"status":"unsupported","error":error})
            );
            std::process::exit(2);
        }
    }
}
