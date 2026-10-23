use std::env;
use std::fs;
use std::path::{Path, PathBuf};

/// The Linux task must explicitly declare its physical output namespace.
/// Windows callers retain the original drive-root policy.
pub fn explicit_linux_output(path: &Path) -> bool {
    if !cfg!(target_os = "linux") {
        return false;
    }
    println!("cargo:rerun-if-env-changed=ZIRCON_LINUX_CARGO_TARGET_ROOT");
    let Some(root) = env::var_os("ZIRCON_LINUX_CARGO_TARGET_ROOT").map(PathBuf::from) else {
        return false;
    };
    assert_eq!(
        root.parent(),
        Some(Path::new("/workspace/cargo-targets")),
        "Linux exception must name one direct child of /workspace/cargo-targets"
    );
    assert_eq!(
        fs::canonicalize(&root).expect("Linux output root must exist"),
        root,
        "Linux output exception must be a physical path without aliases"
    );
    path.starts_with(root)
}

/// Read the pinned native checkout in place, or an explicitly supplied tree.
/// All CMake outputs stay under this crate's OUT_DIR.
pub fn native_source() -> PathBuf {
    println!("cargo:rerun-if-env-changed=ZR_VM_NATIVE_SOURCE_ROOT");
    if let Some(root) = env::var_os("ZR_VM_NATIVE_SOURCE_ROOT") {
        return PathBuf::from(root);
    }
    let cargo_home = env::var_os("CARGO_HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|| {
            PathBuf::from(
                env::var_os("HOME")
                    .or_else(|| env::var_os("USERPROFILE"))
                    .expect("set CARGO_HOME or ZR_VM_NATIVE_SOURCE_ROOT"),
            )
            .join(".cargo")
        });
    let checkouts = cargo_home.join("git/checkouts");
    let mut candidates = Vec::new();
    for entry in fs::read_dir(checkouts).expect("read Cargo native dependency cache") {
        let entry = entry.expect("read Cargo checkout");
        if !entry.file_name().to_string_lossy().starts_with("zr_vm-") {
            continue;
        }
        for revision in fs::read_dir(entry.path()).expect("read ZrVM revisions") {
            let revision = revision.expect("read ZrVM revision");
            if revision.file_name() == "3c1352e" && revision.path().join("CMakeLists.txt").is_file()
            {
                candidates.push(revision.path());
            }
        }
    }
    assert_eq!(
        candidates.len(),
        1,
        "expected one pinned ZrVM native checkout; set ZR_VM_NATIVE_SOURCE_ROOT explicitly"
    );
    candidates.pop().unwrap()
}
