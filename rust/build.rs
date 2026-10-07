fn main() {
    println!("cargo:rerun-if-changed=app.manifest");
    // The Store's WACK requires a DPI-aware executable; the linker flags are MSVC's.
    if std::env::var("CARGO_CFG_TARGET_OS").ok().as_deref() != Some("windows")
        || std::env::var("CARGO_CFG_TARGET_ENV").ok().as_deref() != Some("msvc")
    {
        return;
    }
    let manifest =
        std::path::PathBuf::from(std::env::var("CARGO_MANIFEST_DIR").expect("manifest dir"))
            .join("app.manifest");
    println!("cargo:rustc-link-arg-bin=scalarscope=/MANIFEST:EMBED");
    println!("cargo:rustc-link-arg-bin=scalarscope=/MANIFESTUAC:no");
    println!(
        "cargo:rustc-link-arg-bin=scalarscope=/MANIFESTINPUT:{}",
        manifest.display()
    );
}
