use std::ffi::OsString;
use std::fs;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

#[test]
fn run_rejects_missing_dependency_without_writing_pack() {
    let root = unique_temp_dir("missing-dependency-no-pack");
    let manifest_path = root.join("assets.json");
    let source_path = root.join("main.scene");
    let pack_path = root.join("out").join("assets.zrpack");
    let report_path = root.join("out").join("report.json");
    fs::write(&source_path, b"scene").unwrap();
    fs::write(
        &manifest_path,
        serde_json::json!({
            "roots": ["scenes/main.zscene"],
            "assets": [
                {
                    "path": "scenes/main.zscene",
                    "source": "main.scene",
                    "dependencies": ["textures/missing.png"],
                    "labels": []
                }
            ]
        })
        .to_string(),
    )
    .unwrap();

    let exit_code = super::run([
        os("--profile"),
        os("windows-release"),
        os("--manifest"),
        manifest_path.clone().into_os_string(),
        os("--pack"),
        pack_path.clone().into_os_string(),
        os("--report"),
        report_path.clone().into_os_string(),
    ])
    .unwrap();

    assert_eq!(exit_code, std::process::ExitCode::from(2));
    assert!(!pack_path.exists());
    let report: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(&report_path).unwrap()).unwrap();
    assert_eq!(report["fatal"], true);
    assert!(report["manifest"].is_null());
    assert_eq!(report["asset_count"], 0);
    assert_eq!(report["chunk_count"], 0);
    assert_eq!(
        report["trim_report"]["missing_dependencies"][0]["owner"],
        "scenes/main.zscene"
    );
    assert_eq!(
        report["trim_report"]["missing_dependencies"][0]["dependency"],
        "textures/missing.png"
    );
    assert_eq!(
        report["trim_report"]["duplicate_assets"],
        serde_json::json!([])
    );
    let diagnostics = report["diagnostics"].as_array().unwrap();
    assert!(diagnostics.iter().any(
        |diagnostic| diagnostic == "pack stage stopped because asset dependencies are missing"
    ));

    let _ = fs::remove_dir_all(root);
}

#[test]
fn run_rejects_duplicate_trim_input_without_writing_pack() {
    let root = unique_temp_dir("duplicate-trim-no-pack");
    let manifest_path = root.join("assets.json");
    let source_path = root.join("main.scene");
    let pack_path = root.join("out").join("assets.zrpack");
    let report_path = root.join("out").join("report.json");
    fs::write(&source_path, b"scene").unwrap();
    fs::write(
        &manifest_path,
        serde_json::json!({
            "roots": ["scenes/main.zscene"],
            "assets": [
                {
                    "path": "scenes/main.zscene",
                    "source": source_path,
                    "dependencies": [],
                    "labels": []
                },
                {
                    "path": "scenes/main.zscene",
                    "source": source_path,
                    "dependencies": [],
                    "labels": []
                }
            ]
        })
        .to_string(),
    )
    .unwrap();

    let exit_code = super::run([
        os("--profile"),
        os("windows-release"),
        os("--manifest"),
        manifest_path.clone().into_os_string(),
        os("--pack"),
        pack_path.clone().into_os_string(),
        os("--report"),
        report_path.clone().into_os_string(),
    ])
    .unwrap();

    assert_eq!(exit_code, std::process::ExitCode::from(2));
    assert!(!pack_path.exists());
    let report =
        serde_json::from_str::<serde_json::Value>(&fs::read_to_string(report_path).unwrap())
            .unwrap();
    assert_eq!(report["fatal"], true);
    assert_eq!(report["manifest"], serde_json::Value::Null);
    assert_eq!(report["asset_count"], 0);
    assert_eq!(report["chunk_count"], 0);
    assert_eq!(
        report["trim_report"]["duplicate_assets"],
        serde_json::json!(["scenes/main.zscene"])
    );
    assert_eq!(
        report["diagnostics"],
        serde_json::json!(["asset scenes/main.zscene is duplicated in trim input"])
    );

    let _ = fs::remove_dir_all(root);
}

#[test]
fn run_reports_missing_asset_source_without_writing_pack() {
    let root = unique_temp_dir("missing-source-no-pack");
    let manifest_path = root.join("assets.json");
    let pack_path = root.join("out").join("assets.zrpack");
    let report_path = root.join("out").join("report.json");
    fs::write(
        &manifest_path,
        serde_json::json!({
            "roots": ["scenes/main.zscene"],
            "assets": [
                {
                    "path": "scenes/main.zscene",
                    "source": "missing.scene",
                    "dependencies": [],
                    "labels": []
                }
            ]
        })
        .to_string(),
    )
    .unwrap();

    let exit_code = super::run([
        os("--profile"),
        os("windows-release"),
        os("--manifest"),
        manifest_path.clone().into_os_string(),
        os("--pack"),
        pack_path.clone().into_os_string(),
        os("--report"),
        report_path.clone().into_os_string(),
    ])
    .unwrap();

    assert_eq!(exit_code, std::process::ExitCode::from(2));
    assert!(!pack_path.exists());
    let report: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(&report_path).unwrap()).unwrap();
    assert_eq!(report["fatal"], true);
    assert_eq!(report["manifest"], serde_json::Value::Null);
    assert_eq!(report["asset_count"], 0);
    assert_eq!(report["chunk_count"], 0);
    assert_eq!(
        report["trim_report"]["included_assets"],
        serde_json::json!(["scenes/main.zscene"])
    );
    assert!(report["diagnostics"]
        .as_array()
        .unwrap()
        .iter()
        .any(|diagnostic| diagnostic
            .as_str()
            .unwrap()
            .contains("failed to read asset source")));

    let _ = fs::remove_dir_all(root);
}

fn os(value: impl Into<OsString>) -> OsString {
    value.into()
}

fn unique_temp_dir(label: &str) -> PathBuf {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let root = std::env::temp_dir().join(format!("zircon-export-pack-{label}-{nanos}"));
    fs::create_dir_all(&root).unwrap();
    root
}
