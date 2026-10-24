// SPDX-License-Identifier: GPL-3.0-or-later
fn main() {
    println!("cargo:rerun-if-changed=palette.manifest");
    if std::env::var("CARGO_CFG_TARGET_ENV").as_deref() == Ok("msvc") {
        let manifest = std::path::Path::new(&std::env::var("CARGO_MANIFEST_DIR").unwrap())
            .join("palette.manifest");
        println!("cargo:rustc-link-arg-bin=ipa-symbol-palette=/MANIFEST:EMBED");
        println!(
            "cargo:rustc-link-arg-bin=ipa-symbol-palette=/MANIFESTINPUT:{}",
            manifest.display()
        );
    }
}
