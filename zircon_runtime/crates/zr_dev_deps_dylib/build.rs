fn main() {
    println!("cargo:rerun-if-changed=build.rs");
    if std::env::var_os("CARGO_FEATURE_RUNTIME_ASSETS").is_none()
        || std::env::var("CARGO_CFG_TARGET_ENV").as_deref() != Ok("msvc")
    {
        return;
    }

    // Rust dylibs export Rust items, but Runtime also calls these native entry
    // points directly. Export them from the bundled meshopt static library.
    for symbol in [
        "meshopt_decodeVertexBuffer",
        "meshopt_decodeIndexBuffer",
        "meshopt_decodeIndexSequence",
        "meshopt_decodeFilterOct",
        "meshopt_decodeFilterQuat",
        "meshopt_decodeFilterExp",
        "meshopt_decodeFilterColor",
    ] {
        println!("cargo:rustc-link-arg=/EXPORT:{symbol}");
    }
}
