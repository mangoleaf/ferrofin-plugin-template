//! Exposes the `[package.metadata.ferrofin]` identity from Cargo.toml as
//! compile-time env vars, so `descriptor()` in src/lib.rs and the manifest
//! generator read the SAME values — the runtime id can never drift from the
//! catalog entry. No TOML dependency: the section is simple `key = "value"`
//! lines, parsed by hand.

use std::fmt::Write as _;

fn main() {
    println!("cargo:rerun-if-changed=Cargo.toml");
    let manifest = std::fs::read_to_string("Cargo.toml").expect("read Cargo.toml");
    let mut in_section = false;
    let mut found = String::new();
    for line in manifest.lines() {
        let line = line.trim();
        if line.starts_with('[') {
            in_section = line == "[package.metadata.ferrofin]";
            continue;
        }
        if !in_section || line.starts_with('#') {
            continue;
        }
        if let Some((key, value)) = line.split_once('=') {
            let key = key.trim();
            let value = value.trim().trim_matches('"');
            if matches!(key, "guid" | "name" | "description") {
                println!(
                    "cargo:rustc-env=FERROFIN_PLUGIN_{}={}",
                    key.to_uppercase(),
                    value
                );
                let _ = writeln!(found, "{key}");
            }
        }
    }
    for required in ["guid", "name", "description"] {
        assert!(
            found.contains(required),
            "[package.metadata.ferrofin] must define `{required}` in Cargo.toml"
        );
    }
}
