use super::approved_owned_directory;
use std::path::Path;

#[test]
fn actual_windows_canonical_drive_root_keeps_verbatim_admission() {
    let canonical = [
        "D:/cargo-targets/zircon-local",
        "E:/cargo-targets/zircon-local",
        "F:/cargo-targets/zircon-local",
    ]
    .into_iter()
    .find_map(|root| Path::new(root).canonicalize().ok())
    .expect("Windows validation requires an actual approved physical output root");
    assert!(approved_owned_directory(&canonical), "{canonical:?}");
    assert!(approved_owned_directory(
        &canonical.join("penpot-workbench")
    ));
}

#[test]
fn rejects_unc_other_drives_and_nested_lookalike_roots() {
    for path in [
        r"\\server\share\cargo-targets\zircon-local",
        r"\\?\UNC\server\share\cargo-targets\zircon-local",
        r"C:\cargo-targets\zircon-local",
        r"D:\nested\cargo-targets\zircon-local",
        r"D:\cargo-targets\zircon-local-other",
        r"D:\cargo-targets\zircon-local\..\other",
    ] {
        assert!(!approved_owned_directory(Path::new(path)), "{path}");
    }
    assert!(approved_owned_directory(Path::new(
        r"D:\cargo-targets\zircon-local"
    )));
    assert!(approved_owned_directory(Path::new(
        r"\\?\D:\cargo-targets\zircon-local"
    )));
}
