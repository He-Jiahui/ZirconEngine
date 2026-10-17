const EDITOR_HOST_STACK_RESERVE_BYTES: usize = 8 * 1024 * 1024;

fn main() {
    println!("cargo:rerun-if-changed=build.rs");

    if std::env::var("CARGO_CFG_TARGET_OS").as_deref() == Ok("windows")
        && std::env::var("CARGO_CFG_TARGET_ENV").as_deref() == Ok("msvc")
        && std::env::var_os("CARGO_FEATURE_TARGET_EDITOR_HOST").is_some()
    {
        println!(
            "cargo:rustc-link-arg-bin=zircon_editor=/STACK:{}",
            EDITOR_HOST_STACK_RESERVE_BYTES
        );
    }
}
