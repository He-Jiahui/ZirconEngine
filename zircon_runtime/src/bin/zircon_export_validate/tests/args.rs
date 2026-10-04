use std::ffi::OsString;
use std::fs;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

use super::parse;

#[test]
fn export_validate_closeout_accepts_explicit_stdout_and_contents_artifact() {
    let args = parse([
        OsString::from("--profile"),
        OsString::from("client"),
        OsString::from("--report"),
        OsString::from("out/report.json"),
        OsString::from("--contents-artifact"),
        OsString::from("out/contents.json"),
        OsString::from("--stdout"),
    ])
    .expect("arguments should parse")
    .expect("help should not be requested");

    assert_eq!(args.report, Some(PathBuf::from("out/report.json")));
    assert_eq!(
        args.contents_artifact,
        Some(PathBuf::from("out/contents.json"))
    );
    assert!(args.stdout);
}

#[test]
fn export_validate_closeout_rejects_identical_report_and_contents_artifact_paths() {
    let error = parse([
        OsString::from("--profile"),
        OsString::from("client"),
        OsString::from("--report"),
        OsString::from("out/report.json"),
        OsString::from("--contents-artifact"),
        OsString::from("out/report.json"),
    ])
    .expect_err("the report must not overwrite the contents artifact");

    assert!(error
        .to_string()
        .contains("--report and --contents-artifact must use different paths"));
}

#[test]
fn export_validate_closeout_defers_parent_component_identity_to_output_open() {
    let args = parse([
        OsString::from("--profile"),
        OsString::from("client"),
        OsString::from("--report"),
        OsString::from("out/report.json"),
        OsString::from("--contents-artifact"),
        OsString::from("out/nested/../report.json"),
    ])
    .expect("argument parsing must not guess across symlinked parent components")
    .expect("help should not be requested");

    assert_eq!(args.report, Some(PathBuf::from("out/report.json")));
    assert_eq!(
        args.contents_artifact,
        Some(PathBuf::from("out/nested/../report.json"))
    );
}

#[cfg(windows)]
#[test]
fn export_validate_closeout_rejects_windows_case_aliased_output_paths() {
    let error = parse([
        OsString::from("--profile"),
        OsString::from("client"),
        OsString::from("--report"),
        OsString::from("out/report.json"),
        OsString::from("--contents-artifact"),
        OsString::from("out/REPORT.json"),
    ])
    .expect_err("Windows case aliases must not address both outputs");

    assert!(error
        .to_string()
        .contains("--report and --contents-artifact must use different paths"));
}

#[cfg(windows)]
#[test]
fn export_validate_closeout_rejects_windows_verbatim_output_aliases() {
    let error = parse([
        OsString::from("--profile"),
        OsString::from("client"),
        OsString::from("--report"),
        OsString::from(r"\\?\C:\ZirconBuilds\mvp\report.json"),
        OsString::from("--contents-artifact"),
        OsString::from(r"C:\ZirconBuilds\mvp\report.json"),
    ])
    .expect_err("Windows verbatim aliases must not address both outputs");

    assert!(error
        .to_string()
        .contains("--report and --contents-artifact must use different paths"));
}

#[test]
fn export_validate_closeout_rejects_hard_linked_output_paths() {
    let root = unique_temp_dir("hard-link-alias");
    let report = root.join("report.json");
    let artifact = root.join("contents.json");
    fs::write(&report, "existing output").expect("report fixture should be written");
    fs::hard_link(&report, &artifact).expect("hard-link fixture should be created");

    let error = parse([
        OsString::from("--profile"),
        OsString::from("client"),
        OsString::from("--report"),
        report.into_os_string(),
        OsString::from("--contents-artifact"),
        artifact.into_os_string(),
    ])
    .expect_err("hard links must not address both outputs");

    assert!(error
        .to_string()
        .contains("--report and --contents-artifact must use different paths"));
    let _ = fs::remove_dir_all(root);
}

fn unique_temp_dir(label: &str) -> PathBuf {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system time should follow the Unix epoch")
        .as_nanos();
    let root = std::env::temp_dir().join(format!(
        "zircon-export-validate-args-{label}-{}-{nanos}",
        std::process::id()
    ));
    fs::create_dir_all(&root).expect("temporary test directory should be created");
    root
}
