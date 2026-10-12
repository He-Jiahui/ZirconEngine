use super::super::RelPath;

#[test]
fn relative_path_normalizes_portable_separators() {
    let path = RelPath::parse(r"content\textures//characters").unwrap();

    assert_eq!(path.as_str(), "content/textures/characters");
}

#[test]
fn relative_path_rejects_empty_absolute_dot_parent_and_prefix_forms() {
    for invalid in [
        "",
        "/assets",
        ".",
        "assets/./models",
        "..",
        "assets/../outside",
        "C:/assets",
        "assets/C:/outside",
        "assets\\C:outside",
        "assets/file:stream",
        "assets/CON",
        "assets/con.txt",
        "assets/PRN.log",
        "assets/AUX .data",
        "assets/nul.",
        "assets/COM1",
        "assets/lpt9.txt",
        "assets/COM¹.txt",
        r"\\server\share\assets",
    ] {
        assert!(RelPath::parse(invalid).is_err(), "accepted {invalid:?}");
    }
}
