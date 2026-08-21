use std::ffi::OsString;

use zircon_editor::{EditorGuiStartupRequest, EditorHostRunConfig};
use zircon_runtime::asset::{project::ProjectPaths, AssetUri};
use zircon_runtime_interface::hub_protocol::HubSessionToken;

#[test]
fn first_frame_exit_flag_projects_into_editor_host_config() {
    let startup_request = Some(EditorGuiStartupRequest::open_builtin_view(
        "editor.material_component_lab",
    ));
    let config = super::super::editor_host_run_config_with_first_frame_exit(
        startup_request.clone(),
        None,
        None,
        true,
        None,
    );

    assert_eq!(config.startup_request(), startup_request.as_ref());
    assert!(config.exit_after_first_presented_frame());
}

#[test]
fn editor_host_config_carries_the_optional_first_presented_frame_capture() {
    let capture_path = std::path::PathBuf::from("evidence/editor-first-frame.png");
    let resolved_capture_path = ProjectPaths::resolve_path(&capture_path).unwrap();
    let config = super::super::editor_host_run_config_with_first_frame_exit(
        None,
        None,
        None,
        true,
        Some(resolved_capture_path.clone()),
    );

    assert_eq!(
        config.first_presented_frame_capture_path(),
        Some(&resolved_capture_path)
    );
    assert!(config.exit_after_first_presented_frame());
}

#[test]
fn editor_host_config_carries_a_project_startup_scene() {
    let scene_uri =
        AssetUri::parse("res://scenes/main.scene.toml").expect("scene URI should parse");
    let config = super::super::editor_host_run_config_with_first_frame_exit(
        None,
        Some(scene_uri.clone()),
        None,
        false,
        None,
    );

    assert_eq!(config.startup_scene_uri(), Some(&scene_uri));
}

#[test]
fn editor_host_config_accepts_a_verified_hub_handshake_from_the_app_boundary() {
    let session = "0d9a5890-0e44-4e2a-b77e-3e5d4fdf1e52"
        .parse::<HubSessionToken>()
        .expect("valid Hub session");

    let config = EditorHostRunConfig::new().with_hub_handshake("E:/Projects/My Game", session);

    assert_eq!(config.startup_request(), None);
    assert!(!config.exit_after_first_presented_frame());
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
fn editor_first_frame_capture_path_resolves_a_relative_environment_value() {
    let path = std::path::PathBuf::from("captures/editor-first-frame.png");

    assert_eq!(
        super::super::editor_first_frame_capture_path_from_value(None).unwrap(),
        None
    );
    assert_eq!(
        super::super::editor_first_frame_capture_path_from_value(Some(
            path.clone().into_os_string()
        ))
        .unwrap(),
        Some(ProjectPaths::resolve_path(&path).unwrap())
    );
}

#[test]
fn editor_first_frame_capture_path_resolves_an_absolute_environment_value() {
    for absolute in [
        std::path::PathBuf::from(r"C:\zircon\editor-first-frame.png"),
        std::path::PathBuf::from(r"\\server\share\editor-first-frame.png"),
    ] {
        assert_eq!(
            super::super::editor_first_frame_capture_path_from_value(Some(
                absolute.clone().into_os_string()
            ))
            .unwrap(),
            Some(ProjectPaths::resolve_path(absolute).unwrap())
        );
    }
}

#[cfg(windows)]
#[test]
fn editor_first_frame_capture_path_rejects_windows_drive_relative_input() {
    assert!(
        super::super::editor_first_frame_capture_path_from_value(Some(OsString::from(
            r"C:editor-first-frame.png",
        )))
        .is_err()
    );
}

#[cfg(unix)]
#[test]
fn editor_first_frame_capture_path_preserves_non_utf8_absolute_path() {
    use std::os::unix::ffi::OsStringExt;

    let value = OsString::from_vec(vec![b'/', b't', b'm', b'p', b'/', 0xFF]);

    assert_eq!(
        super::super::editor_first_frame_capture_path_from_value(Some(value.clone())).unwrap(),
        Some(ProjectPaths::resolve_path(std::path::PathBuf::from(value)).unwrap())
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
        assert!(error.to_string().contains("writable PNG path"));
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
