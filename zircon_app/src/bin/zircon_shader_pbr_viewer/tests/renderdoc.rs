use super::RenderDocCaptureReport;
use crate::work_paths::viewer_test_artifact_root;

fn temporary_capture_path(extension: &str) -> std::path::PathBuf {
    let nonce = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .expect("system time must be after the Unix epoch")
        .as_nanos();
    viewer_test_artifact_root("renderdoc-capture-evidence")
        .join(format!("capture-{nonce}.{extension}"))
}

fn remove_capture_fixture(path: &std::path::Path) {
    let _ = std::fs::remove_file(path);
    std::fs::remove_dir_all(path.parent().expect("test path should have a parent"))
        .expect("test artifact root should be removed");
}

#[test]
fn renderdoc_capture_report_rejects_an_empty_capture_list() {
    let report = RenderDocCaptureReport {
        capture_count: 0,
        latest_capture_path: None,
    };

    let error = report
        .capture_path_for_evidence()
        .expect_err("a completed bridge capture requires an actual RenderDoc record");

    assert!(error.contains("did not record a capture"));
}

#[test]
fn renderdoc_capture_report_rejects_a_missing_latest_path() {
    let report = RenderDocCaptureReport {
        capture_count: 1,
        latest_capture_path: None,
    };

    let error = report
        .capture_path_for_evidence()
        .expect_err("a completed bridge capture requires the latest capture path");

    assert!(error.contains("without a latest capture path"));
}

#[test]
fn renderdoc_capture_report_rejects_a_missing_capture_artifact() {
    let capture_path = temporary_capture_path("rdc");
    let report = RenderDocCaptureReport {
        capture_count: 1,
        latest_capture_path: Some(capture_path),
    };

    let error = report
        .capture_path_for_evidence()
        .expect_err("a missing RenderDoc capture must not be accepted as evidence");

    assert!(error.contains("artifact is unavailable"));
    remove_capture_fixture(
        report
            .latest_capture_path()
            .expect("fixture path should exist"),
    );
}

#[test]
fn renderdoc_capture_report_rejects_non_lowercase_rdc_artifacts() {
    let capture_path = temporary_capture_path("RDC");
    std::fs::write(&capture_path, b"RenderDoc capture").expect("write capture fixture");
    let report = RenderDocCaptureReport {
        capture_count: 1,
        latest_capture_path: Some(capture_path.clone()),
    };

    let result = report.capture_path_for_evidence();
    remove_capture_fixture(&capture_path);

    let error = result.expect_err("only lowercase .rdc files are valid capture evidence");
    assert!(error.contains("lowercase .rdc artifact"));
}

#[test]
fn renderdoc_capture_report_rejects_empty_capture_artifacts() {
    let capture_path = temporary_capture_path("rdc");
    std::fs::write(&capture_path, b"").expect("write empty capture fixture");
    let report = RenderDocCaptureReport {
        capture_count: 1,
        latest_capture_path: Some(capture_path.clone()),
    };

    let result = report.capture_path_for_evidence();
    remove_capture_fixture(&capture_path);

    let error = result.expect_err("an empty RenderDoc capture must not be accepted as evidence");
    assert!(error.contains("artifact is empty"));
}

#[test]
fn renderdoc_capture_report_preserves_an_existing_nonempty_rdc_evidence_path() {
    let capture_path = temporary_capture_path("rdc");
    std::fs::write(&capture_path, b"RenderDoc capture").expect("write capture fixture");
    let report = RenderDocCaptureReport {
        capture_count: 1,
        latest_capture_path: Some(capture_path.clone()),
    };

    let result = report
        .capture_path_for_evidence()
        .map(std::path::Path::to_path_buf);
    remove_capture_fixture(&capture_path);

    assert_eq!(
        result.expect("a completed bridge capture should expose its artifact path"),
        capture_path
    );
}

#[test]
fn renderdoc_api_prefix_covers_template_and_capture_report_entries() {
    assert_eq!(
        std::mem::size_of::<super::RenderDocApi141>(),
        std::mem::size_of::<usize>() * 15,
        "RenderDoc API 1.4.1 needs 11 leading entries plus template, template query, count, and capture pointers"
    );
}

#[test]
fn renderdoc_capture_template_identity_strips_only_the_lowercase_rdc_extension() {
    assert_eq!(
        super::capture_template_identity(std::path::Path::new("E:/evidence/pbr-frame.rdc")),
        std::path::PathBuf::from("E:/evidence/pbr-frame")
    );
    assert_eq!(
        super::capture_template_identity(std::path::Path::new("E:/evidence/pbr-frame.RDC")),
        std::path::PathBuf::from("E:/evidence/pbr-frame.RDC")
    );
    assert_eq!(
        super::capture_template_identity(std::path::Path::new("E:/evidence/pbr-frame.log")),
        std::path::PathBuf::from("E:/evidence/pbr-frame.log")
    );
}

#[test]
fn renderdoc_capture_template_identity_requires_the_applied_template() {
    let expected = std::path::Path::new("E:/evidence/pbr-frame");

    assert!(super::capture_template_matches(
        expected,
        std::path::Path::new("E:/evidence/pbr-frame")
    ));
    assert!(super::capture_template_matches(
        expected,
        std::path::Path::new("e:\\Evidence\\PBR-FRAME")
    ));
    assert!(!super::capture_template_matches(
        expected,
        std::path::Path::new("E:/other/pbr-frame")
    ));
}
