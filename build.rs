fn main() {
    // Generate C header for FFI using cbindgen
    cbindgen::Builder::new()
        .with_crate(".")
        .with_language(cbindgen::Language::C)
        .with_pragma_once(true)
        .with_include_guard("SHIKENMATRIX_H")
        .generate()
        .expect("Unable to generate bindings")
        .write_to_file("shikenmatrix.h");

    #[cfg(target_os = "windows")]
    if std::env::var("CARGO_CFG_TARGET_OS").ok().as_deref() == Some("windows") {
        embed_resource::compile("app-icon.rc", embed_resource::NONE);
    }
}
