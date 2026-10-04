use super::{library_embed_command, target_for_mode, RuntimeTargetMode};

#[test]
fn preallocated_library_embed_command_preserves_contract() {
    let target = target_for_mode(RuntimeTargetMode::ClientRuntime);
    let manifest_path = "Cargo.toml".to_string();
    let target_dir = "stages/compile_host/target".to_string();

    assert_eq!(
        library_embed_command(&target, manifest_path, target_dir, true),
        vec![
            "cargo".to_string(),
            "build".to_string(),
            "--manifest-path".to_string(),
            "Cargo.toml".to_string(),
            "-p".to_string(),
            "zircon_app".to_string(),
            "--bin".to_string(),
            "zircon_runtime".to_string(),
            "--no-default-features".to_string(),
            "--features".to_string(),
            "target-client".to_string(),
            "--target-dir".to_string(),
            "stages/compile_host/target".to_string(),
            "--release".to_string(),
        ]
    );
}
