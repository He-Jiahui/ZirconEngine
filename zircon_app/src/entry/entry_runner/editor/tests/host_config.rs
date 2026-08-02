use std::ffi::OsString;

use zircon_editor::EditorGuiStartupRequest;

#[test]
fn first_frame_exit_flag_projects_into_editor_host_config() {
    let startup_request = Some(EditorGuiStartupRequest::open_builtin_view(
        "editor.material_component_lab",
    ));
    let config = super::super::editor_host_run_config_with_first_frame_exit(
        startup_request.clone(),
        true,
        None,
    );

    assert_eq!(config.startup_request(), startup_request.as_ref());
    assert!(config.exit_after_first_presented_frame());
}

#[test]
fn editor_host_config_carries_the_optional_first_presented_frame_capture() {
    let capture_path = std::path::PathBuf::from("evidence/editor-first-frame.png");
    let config = super::super::editor_host_run_config_with_first_frame_exit(
        None,
        true,
        Some(capture_path.clone()),
    );

    assert_eq!(
        config.first_presented_frame_capture_path(),
        Some(capture_path.as_path())
    );
    assert!(config.exit_after_first_presented_frame());
}

#[test]
fn editor_first_frame_exit_requires_an_explicit_enabled_value() {
    assert!(!super::super::editor_exit_after_first_frame_enabled_value(
        None
    ));
    assert!(!super::super::editor_exit_after_first_frame_enabled_value(
        Some("")
    ));
    assert!(!super::super::editor_exit_after_first_frame_enabled_value(
        Some("0")
    ));
    assert!(!super::super::editor_exit_after_first_frame_enabled_value(
        Some("false")
    ));
    assert!(super::super::editor_exit_after_first_frame_enabled_value(
        Some("1")
    ));
    assert!(super::super::editor_exit_after_first_frame_enabled_value(
        Some("TRUE")
    ));
    assert!(super::super::editor_exit_after_first_frame_enabled_value(
        Some("yes")
    ));
}

#[test]
fn editor_first_frame_capture_path_accepts_an_absolute_environment_value() {
    let path = std::path::absolute("editor-first-frame.png").unwrap();

    assert_eq!(
        super::super::editor_first_frame_capture_path_from_value(None).unwrap(),
        None
    );
    assert_eq!(
        super::super::editor_first_frame_capture_path_from_value(Some(
            path.clone().into_os_string()
        ))
        .unwrap(),
        Some(path)
    );
}

#[test]
fn editor_first_frame_capture_path_rejects_a_relative_environment_value() {
    let error = super::super::editor_first_frame_capture_path_from_value(Some(OsString::from(
        "captures/editor-first-frame.png",
    )))
    .expect_err("relative capture path must not depend on the process working directory");

    assert_eq!(error.kind(), std::io::ErrorKind::InvalidInput);
    assert_eq!(
        error.to_string(),
        "editor startup diagnostic: component=editor_host requested=ZIRCON_EDITOR_CAPTURE_FIRST_FRAME_PNG=captures/editor-first-frame.png cause=first-frame PNG capture path must be absolute recovery=set ZIRCON_EDITOR_CAPTURE_FIRST_FRAME_PNG to a writable absolute PNG path or unset it"
    );
}

#[cfg(windows)]
#[test]
fn editor_first_frame_capture_path_preserves_windows_absolute_path_semantics() {
    for absolute in [
        std::path::PathBuf::from(r"C:\zircon\editor-first-frame.png"),
        std::path::PathBuf::from(r"\\server\share\editor-first-frame.png"),
    ] {
        assert_eq!(
            super::super::editor_first_frame_capture_path_from_value(Some(
                absolute.clone().into_os_string()
            ))
            .unwrap(),
            Some(absolute)
        );
    }

    for relative in [
        OsString::from(r"C:editor-first-frame.png"),
        OsString::from(r"\editor-first-frame.png"),
        OsString::from(r"/editor-first-frame.png"),
    ] {
        assert!(super::super::editor_first_frame_capture_path_from_value(Some(relative)).is_err());
    }
}

#[cfg(unix)]
#[test]
fn editor_first_frame_capture_path_preserves_non_utf8_absolute_path() {
    use std::os::unix::ffi::OsStringExt;

    let value = OsString::from_vec(vec![b'/', b't', b'm', b'p', b'/', 0xFF]);

    assert_eq!(
        super::super::editor_first_frame_capture_path_from_value(Some(value.clone())).unwrap(),
        Some(std::path::PathBuf::from(value))
    );
}

#[test]
fn editor_first_frame_capture_path_rejects_an_explicit_empty_or_blank_environment_value() {
    for value in [
        OsString::new(),
        OsString::from(" "),
        OsString::from("\u{2003}"),
    ] {
        let error =
            super::super::editor_first_frame_capture_path_from_value(Some(value)).unwrap_err();

        assert_eq!(error.kind(), std::io::ErrorKind::InvalidInput);
        assert!(error
            .to_string()
            .contains("ZIRCON_EDITOR_CAPTURE_FIRST_FRAME_PNG"));
        assert!(error.to_string().contains("writable absolute PNG path"));
    }
}

#[test]
fn editor_validates_the_first_frame_capture_path_before_project_startup() {
    let source = include_str!("../../editor.rs");
    let capture_validation = source
        .find("let first_frame_capture_path = editor_first_frame_capture_path()?;")
        .expect("editor startup should validate the optional capture path first");
    let prepare_startup = source
        .find("let prepared_startup = prepare_editor_gui_startup(gui_startup_request).map_err(")
        .expect("editor startup should prepare the requested project after input validation");

    assert!(
        capture_validation < prepare_startup,
        "invalid first-frame capture configuration must fail before project creation or opening"
    );
}
