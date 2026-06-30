use std::path::Path;

fn main() {
    // Ensure the panel dist directory exists at compile time so that
    // `rust-embed` can embed it (in release the panel is built first via the
    // monorepo workspace dependency; in dev it is served by Vite instead).
    let manifest_dir = std::env::var("CARGO_MANIFEST_DIR").unwrap();
    let dist = Path::new(&manifest_dir).join("../panel/dist");
    if !dist.exists() {
        std::fs::create_dir_all(&dist).ok();
    }
    println!("cargo:rerun-if-changed=../panel/dist");
}
