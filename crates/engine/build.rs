//! Embeds the Windows application manifest in the crate's executables.

use embed_manifest::manifest::DpiAwareness;
use embed_manifest::{embed_manifest, new_manifest};

fn main() {
    println!("cargo::rerun-if-changed=build.rs");
    if std::env::var_os("CARGO_CFG_WINDOWS").is_some() {
        // Per-Monitor V2 is what makes every coordinate Windows hands the backend a
        // physical pixel, whatever each Monitor's scale.
        let manifest = new_manifest("views").dpi_awareness(DpiAwareness::PerMonitorV2Only);
        embed_manifest(manifest).expect("the manifest should embed");
    }
}
