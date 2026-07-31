use super::super::{ProjectNameError, validate_project_name};

#[test]
fn project_name_accepts_one_portable_filename_component() {
    for valid in ["Game", "My Game", "项目", "game.project"] {
        assert_eq!(validate_project_name(valid), Ok(()), "rejected {valid:?}");
    }
}

#[test]
fn project_name_rejects_paths_prefixes_reserved_names_and_windows_tail_aliases() {
    for invalid in [
        "",
        " ",
        ".",
        "..",
        "../Game",
        "folder/Game",
        r"folder\Game",
        "C:Game",
        "C:/Game",
        r"\\server\share",
        "Game.",
        "Game ",
        " Game",
        "CON",
        "con.txt",
        "LPT1",
        "COM9.log",
        "bad:name",
        "bad*name",
    ] {
        assert!(
            validate_project_name(invalid).is_err(),
            "accepted unsafe project name {invalid:?}"
        );
    }
    assert!(matches!(
        validate_project_name("CON"),
        Err(ProjectNameError::WindowsReserved { .. })
    ));
}
