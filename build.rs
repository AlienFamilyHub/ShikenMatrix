fn main() {
    // Set install name for macOS dylib
    #[cfg(target_os = "macos")]
    {
        println!("cargo:rustc-cdylib-link-arg=-Wl,-install_name,@rpath/libshikenmatrix.dylib");
    }

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
        let _ = embed_resource::compile("app-icon.rc", embed_resource::NONE);
    }
}
