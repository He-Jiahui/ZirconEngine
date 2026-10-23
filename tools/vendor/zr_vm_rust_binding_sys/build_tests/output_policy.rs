use super::*;

#[cfg(target_os = "linux")]
#[test]
fn explicit_linux_policy_keeps_outputs_physical_and_scoped() {
    let root = PathBuf::from(env::var_os("ZIRCON_LINUX_CARGO_TARGET_ROOT").unwrap());
    validate_approved_path(&root, "test root");
    validate_approved_path(&root.join("tmp"), "test tmp");
    for outside in [
        PathBuf::from("/tmp"),
        PathBuf::from("target"),
        PathBuf::from("/workspace/ZirconEngine-orbrun/target"),
        root.with_file_name("ze-testgame-lookalike"),
        PathBuf::from("/workspace/nested/cargo-targets/ze-testgame"),
    ] {
        assert!(
            std::panic::catch_unwind(|| validate_approved_path(&outside, "outside")).is_err(),
            "outside path admitted: {}",
            outside.display()
        );
    }
    let fixture = root
        .join("tmp")
        .join(format!("sys-policy-{}", std::process::id()));
    fs::create_dir_all(&fixture).unwrap();
    let alias = fixture.join("alias");
    std::os::unix::fs::symlink(root.join("tmp"), &alias).unwrap();
    assert!(std::panic::catch_unwind(|| validate_approved_path(&alias, "alias")).is_err());
    fs::remove_file(alias).unwrap();
    let real = fixture.join("case");
    fs::create_dir_all(&real).unwrap();
    let folded_alias = fixture.join("CASE");
    std::os::unix::fs::symlink(&real, &folded_alias).unwrap();
    assert!(
        std::panic::catch_unwind(|| validate_approved_path(&folded_alias, "case alias")).is_err()
    );
    fs::remove_file(folded_alias).unwrap();
}
