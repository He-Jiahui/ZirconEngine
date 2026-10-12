use super::*;

fn fixture(label: &str) -> (PathBuf, AuxiliarySourceResolver) {
    let root = std::env::temp_dir().join(format!(
        "zircon-admitted-{label}-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    fs::create_dir_all(root.join("assets/member")).unwrap();
    fs::create_dir_all(root.join("outside")).unwrap();
    fs::write(root.join("assets/member/data.bin"), b"admitted bytes").unwrap();
    fs::write(root.join("outside/data.bin"), b"outside bytes").unwrap();
    let resolver = AuxiliarySourceResolver::for_source(
        &root.join("assets/model.gltf"),
        &crate::asset::AssetUri::parse("res://model.gltf").unwrap(),
        &root.join("assets"),
    )
    .unwrap();
    (root, resolver)
}

fn replace_parent(root: &Path) {
    fs::rename(root.join("assets/member"), root.join("assets/original")).unwrap();
    #[cfg(windows)]
    {
        let output = std::process::Command::new("cmd")
            .args(["/C", "mklink", "/J"])
            .arg(root.join("assets/member"))
            .arg(root.join("outside"))
            .output()
            .unwrap();
        assert!(
            output.status.success(),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
    }
    #[cfg(unix)]
    std::os::unix::fs::symlink(root.join("outside"), root.join("assets/member")).unwrap();
}

fn cleanup(root: &Path) {
    #[cfg(windows)]
    fs::remove_dir(root.join("assets/member")).unwrap();
    #[cfg(unix)]
    fs::remove_file(root.join("assets/member")).unwrap();
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn malformed_percent_escape_is_rejected_before_path_resolution() {
    let (root, resolver) = fixture("malformed-percent");
    for reference in ["member/%", "member/%2", "member/%GG"] {
        let error = resolver
            .resolve_gltf_uri_lexical(reference)
            .expect_err("malformed URI escapes must fail closed");
        assert!(
            error.to_string().contains("malformed percent escape"),
            "{reference}: {error}"
        );
    }
    cleanup(&root);
}

#[test]
#[cfg(any(windows, target_os = "linux"))]
fn auxiliary_snapshot_rejects_parent_replacement_between_admission_and_open() {
    let (root, resolver) = fixture("before-open");
    let result = resolver.read_path_snapshot_with_hooks(
        Path::new("member/data.bin"),
        1024,
        || replace_parent(&root),
        || {},
    );
    assert!(result.is_err(), "an outside handle must never be admitted");
    cleanup(&root);
}

#[test]
#[cfg(any(windows, target_os = "linux"))]
fn auxiliary_snapshot_reads_original_handle_after_parent_replacement() {
    let (root, resolver) = fixture("after-open");
    let (_, bytes, _) = resolver
        .read_path_snapshot_with_hooks(
            Path::new("member/data.bin"),
            1024,
            || {},
            || replace_parent(&root),
        )
        .unwrap();
    assert_eq!(bytes, b"admitted bytes");
    cleanup(&root);
}
