const EDITOR_HOST_STACK_RESERVE_BYTES: usize = 8 * 1024 * 1024;

fn main() {
    println!("cargo:rerun-if-changed=build.rs");

    if cfg!(all(windows, target_env = "msvc")) {
        println!(
            "cargo:rustc-link-arg-bin=zircon_editor=/STACK:{}",
            EDITOR_HOST_STACK_RESERVE_BYTES
        );
    }
}
