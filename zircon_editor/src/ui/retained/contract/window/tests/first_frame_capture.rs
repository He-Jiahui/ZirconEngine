use super::super::UiHostWindow;
use crate::ui::retained_host::host_contract::diagnostics::HostWindowDiagnosticSeverity;
use std::time::{SystemTime, UNIX_EPOCH};
use zircon_runtime::asset::project::ProjectPaths;

#[test]
fn first_presented_frame_capture_writes_one_png_and_consumes_its_request() {
    let host = UiHostWindow::new().expect("host window should construct for capture test");
    let path = std::env::temp_dir().join(format!(
        "zircon-editor-first-presented-frame-{}-{}.png",
        std::process::id(),
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system clock should be after the Unix epoch")
            .as_nanos()
    ));
    let _ = std::fs::remove_file(&path);

    let resolved_path = ProjectPaths::resolve_path(&path).unwrap();
    host.set_first_presented_frame_capture_path(Some(resolved_path.clone()));

    let written = host
        .capture_first_presented_frame()
        .expect("first frame capture should succeed");
    assert_eq!(written, Some(resolved_path));
    let png = std::fs::read(&path).expect("capture should create a PNG");
    assert!(png.starts_with(b"\x89PNG\r\n\x1a\n"));
    assert_eq!(
        host.capture_first_presented_frame()
            .expect("capture request should be consumed"),
        None
    );
    let diagnostics = host.take_host_diagnostics();
    assert_eq!(diagnostics.len(), 1);
    assert_eq!(
        diagnostics[0].severity(),
        HostWindowDiagnosticSeverity::Info
    );
    assert!(diagnostics[0]
        .message()
        .contains("editor_product_frame_capture_written"));

    std::fs::remove_file(path).expect("capture artifact should be removable");
}

#[test]
fn first_presented_frame_capture_reports_an_unwritable_parent_to_the_app_boundary() {
    let host = UiHostWindow::new().expect("host window should construct for capture test");
    let parent_file = std::env::temp_dir().join(format!(
        "zircon-editor-first-presented-frame-parent-{}-{}",
        std::process::id(),
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system clock should be after the Unix epoch")
            .as_nanos()
    ));
    let _ = std::fs::remove_file(&parent_file);
    std::fs::write(&parent_file, b"not a directory")
        .expect("capture test should create a blocking parent file");
    host.set_first_presented_frame_capture_path(Some(
        ProjectPaths::resolve_path(parent_file.join("capture.png")).unwrap(),
    ));

    let error = host
        .capture_first_presented_frame()
        .expect_err("a file cannot become the capture directory");
    assert!(error
        .to_string()
        .contains("failed to create editor first-frame capture directory"));
    host.record_first_presented_frame_capture_error(&error);
    assert_eq!(
        host.take_first_presented_frame_capture_error(),
        Some(error.to_string())
    );

    std::fs::remove_file(parent_file).expect("blocking capture parent should be removable");
}
