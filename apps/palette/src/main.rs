// SPDX-License-Identifier: GPL-3.0-or-later
#![cfg_attr(windows, windows_subsystem = "windows")]
#[cfg(windows)]
mod windows;
fn main() {
    let result = run();
    if let Err(error) = result {
        #[cfg(windows)]
        windows::error(&error);
        #[cfg(not(windows))]
        eprintln!("{error}");
        std::process::exit(1);
    }
}
fn run() -> Result<(), String> {
    let catalog = symbol_catalog::Catalog::bundled()?;
    let args: Vec<String> = std::env::args().skip(1).collect();
    if args.first().is_some_and(|a| a == "--check") {
        // Machine-readable self-check never initializes a model, network or UI.
        let output = args
            .get(1)
            .ok_or("Usage: ipa-symbol-palette --check output.json")?;
        let value = serde_json::json!({"version":env!("CARGO_PKG_VERSION"),"symbols":catalog.symbols.len(),"model_required":false,"network_required":false,"tie":catalog.search("t͡ʃ",None,None)[0].text,"supplementary_utf16":ipa_palette::utf16_packets("\u{1df00}").len()});
        return std::fs::write(output, value.to_string()).map_err(|e| e.to_string());
    }
    #[cfg(windows)]
    {
        windows::run(catalog, &args)
    }
    #[cfg(not(windows))]
    {
        Err("The native palette currently targets Windows x64 only".into())
    }
}
