use serde::Serialize;
use std::ffi::OsString;
use std::fs;
use std::path::Path;
use std::process::ExitCode;

use crate::pack::{
    ZrPackDeltaDocumentManifest, ZrPackDeltaReader, ZrPackDeltaWriter, ZrPackDocumentManifest,
    ZrPackInputAsset, ZrPackReader, ZrPackTrimReport, ZrPackWriter,
};
use crate::plugin::ExportPipelineStage;

use super::args::{parse, usage};
use super::manifest::ExportAssetPackManifest;

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct ExportPackReport {
    pub stage: ExportPipelineStage,
    pub profile: String,
    pub asset_manifest: String,
    pub pack: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub previous_pack: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub delta_pack: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub stage_output: Option<String>,
    pub fatal: bool,
    pub diagnostics: Vec<String>,
    pub trim_report: ZrPackTrimReport,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub manifest: Option<ZrPackDocumentManifest>,
    pub asset_count: usize,
    pub chunk_count: usize,
    pub deduplicated_assets: Vec<String>,
    pub deterministic_double_run: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub delta_manifest: Option<ZrPackDeltaDocumentManifest>,
    pub delta_asset_count: usize,
    pub delta_chunk_count: usize,
    pub delta_removed_assets: Vec<String>,
    pub delta_reused_assets: Vec<String>,
    pub delta_apply_verified: bool,
}

pub fn run(args: impl IntoIterator<Item = OsString>) -> Result<ExitCode, String> {
    let Some(args) = parse(args)? else {
        println!("{}", usage("zircon export pack writer"));
        return Ok(ExitCode::SUCCESS);
    };

    let manifest_dir = args.manifest.parent().unwrap_or_else(|| Path::new("."));
    let asset_manifest_text = fs::read_to_string(&args.manifest).map_err(|error| {
        format!(
            "failed to read asset pack manifest {}: {error}",
            args.manifest.display()
        )
    })?;
    let asset_manifest = serde_json::from_str::<ExportAssetPackManifest>(&asset_manifest_text)
        .map_err(|error| format!("failed to decode asset pack manifest: {error}"))?;
    let pack_inputs = asset_manifest.pack_inputs(manifest_dir)?;
    let mut diagnostics = pack_inputs.diagnostics.clone();
    let fatal_preflight = pack_inputs.trim_report.has_missing_dependencies()
        || pack_inputs.trim_report.has_duplicate_assets()
        || !pack_inputs.asset_source_errors.is_empty();

    let mut report = if fatal_preflight {
        ExportPackReport {
            stage: ExportPipelineStage::Pack,
            profile: args.profile.clone(),
            asset_manifest: args.manifest.display().to_string(),
            pack: args.pack.display().to_string(),
            previous_pack: args
                .previous_pack
                .as_ref()
                .map(|path| path.display().to_string()),
            delta_pack: args
                .delta_pack
                .as_ref()
                .map(|path| path.display().to_string()),
            stage_output: args
                .stage_output
                .as_ref()
                .map(|path| path.display().to_string()),
            fatal: true,
            diagnostics,
            asset_count: 0,
            chunk_count: 0,
            deduplicated_assets: Vec::new(),
            trim_report: pack_inputs.trim_report,
            manifest: None,
            deterministic_double_run: false,
            delta_manifest: None,
            delta_asset_count: 0,
            delta_chunk_count: 0,
            delta_removed_assets: Vec::new(),
            delta_reused_assets: Vec::new(),
            delta_apply_verified: false,
        }
    } else {
        match ZrPackWriter::write(pack_inputs.pack_assets.clone()) {
            Ok(write_report) => {
                let deterministic_double_run = deterministic_double_run(
                    args.determinism_check,
                    &pack_inputs.pack_assets,
                    &write_report.bytes,
                    &mut diagnostics,
                )?;
                if let Some(parent) = args.pack.parent() {
                    if !parent.as_os_str().is_empty() {
                        fs::create_dir_all(parent).map_err(|error| {
                            format!(
                                "failed to create pack directory {}: {error}",
                                parent.display()
                            )
                        })?;
                    }
                }
                fs::write(&args.pack, &write_report.bytes).map_err(|error| {
                    format!("failed to write pack {}: {error}", args.pack.display())
                })?;
                let delta_report = write_delta_pack_if_requested(&args, &write_report.bytes)?;
                ExportPackReport {
                    stage: ExportPipelineStage::Pack,
                    profile: args.profile.clone(),
                    asset_manifest: args.manifest.display().to_string(),
                    pack: args.pack.display().to_string(),
                    previous_pack: args
                        .previous_pack
                        .as_ref()
                        .map(|path| path.display().to_string()),
                    delta_pack: args
                        .delta_pack
                        .as_ref()
                        .map(|path| path.display().to_string()),
                    stage_output: args
                        .stage_output
                        .as_ref()
                        .map(|path| path.display().to_string()),
                    fatal: false,
                    diagnostics,
                    asset_count: write_report.manifest.assets.len(),
                    chunk_count: write_report.manifest.pack.chunks.len(),
                    deduplicated_assets: write_report.deduplicated_assets,
                    trim_report: pack_inputs.trim_report,
                    manifest: Some(write_report.manifest),
                    deterministic_double_run,
                    delta_asset_count: delta_report
                        .as_ref()
                        .map(|report| report.changed_assets.len())
                        .unwrap_or(0),
                    delta_chunk_count: delta_report
                        .as_ref()
                        .map(|report| report.manifest.chunks.len())
                        .unwrap_or(0),
                    delta_removed_assets: delta_report
                        .as_ref()
                        .map(|report| report.removed_assets.clone())
                        .unwrap_or_default(),
                    delta_reused_assets: delta_report
                        .as_ref()
                        .map(|report| report.reused_assets.clone())
                        .unwrap_or_default(),
                    delta_apply_verified: delta_report
                        .as_ref()
                        .map(|report| report.apply_verified)
                        .unwrap_or(false),
                    delta_manifest: delta_report.map(|report| report.manifest),
                }
            }
            Err(error) => ExportPackReport {
                stage: ExportPipelineStage::Pack,
                profile: args.profile.clone(),
                asset_manifest: args.manifest.display().to_string(),
                pack: args.pack.display().to_string(),
                previous_pack: args
                    .previous_pack
                    .as_ref()
                    .map(|path| path.display().to_string()),
                delta_pack: args
                    .delta_pack
                    .as_ref()
                    .map(|path| path.display().to_string()),
                stage_output: args
                    .stage_output
                    .as_ref()
                    .map(|path| path.display().to_string()),
                fatal: true,
                diagnostics: vec![format!("failed to write zrpack: {error}")],
                asset_count: 0,
                chunk_count: 0,
                deduplicated_assets: Vec::new(),
                trim_report: pack_inputs.trim_report,
                manifest: None,
                deterministic_double_run: false,
                delta_manifest: None,
                delta_asset_count: 0,
                delta_chunk_count: 0,
                delta_removed_assets: Vec::new(),
                delta_reused_assets: Vec::new(),
                delta_apply_verified: false,
            },
        }
    };

    if report.trim_report.has_missing_dependencies()
        || report.trim_report.has_duplicate_assets()
        || (args.determinism_check && !report.deterministic_double_run)
        || (report.delta_pack.is_some() && !report.delta_apply_verified)
    {
        report.fatal = true;
    }

    let json = if args.pretty {
        serde_json::to_string_pretty(&report)
    } else {
        serde_json::to_string(&report)
    }
    .map_err(|error| format!("failed to encode export pack report: {error}"))?;

    if let Some(report_path) = &args.report {
        if let Some(parent) = report_path.parent() {
            if !parent.as_os_str().is_empty() {
                fs::create_dir_all(parent).map_err(|error| {
                    format!(
                        "failed to create export pack report directory {}: {error}",
                        parent.display()
                    )
                })?;
            }
        }
        fs::write(report_path, &json).map_err(|error| {
            format!(
                "failed to write export pack report {}: {error}",
                report_path.display()
            )
        })?;
    }

    println!("{json}");
    if report.fatal {
        Ok(ExitCode::from(2))
    } else {
        Ok(ExitCode::SUCCESS)
    }
}

fn write_delta_pack_if_requested(
    args: &super::args::PackArgs,
    target_pack_bytes: &[u8],
) -> Result<Option<VerifiedDeltaWriteReport>, String> {
    let Some(previous_pack) = &args.previous_pack else {
        return Ok(None);
    };
    let Some(delta_pack) = &args.delta_pack else {
        return Ok(None);
    };
    let previous_bytes = fs::read(previous_pack).map_err(|error| {
        format!(
            "failed to read previous pack {}: {error}",
            previous_pack.display()
        )
    })?;
    let base = ZrPackReader::from_bytes(previous_bytes)
        .map_err(|error| format!("failed to read previous zrpack: {error}"))?;
    let target = ZrPackReader::from_bytes(target_pack_bytes.to_vec())
        .map_err(|error| format!("failed to read newly written zrpack: {error}"))?;
    let delta_report = ZrPackDeltaWriter::write(&base, &target)
        .map_err(|error| format!("failed to write delta zrpack: {error}"))?;
    if let Some(parent) = delta_pack.parent() {
        if !parent.as_os_str().is_empty() {
            fs::create_dir_all(parent).map_err(|error| {
                format!(
                    "failed to create delta pack directory {}: {error}",
                    parent.display()
                )
            })?;
        }
    }
    fs::write(delta_pack, &delta_report.bytes).map_err(|error| {
        format!(
            "failed to write delta pack {}: {error}",
            delta_pack.display()
        )
    })?;
    let delta_reader = ZrPackDeltaReader::from_bytes(delta_report.bytes.clone())
        .map_err(|error| format!("failed to verify written delta zrpack: {error}"))?;
    if let Some(asset) = delta_report.changed_assets.first() {
        let _ = delta_reader
            .read_changed_asset(asset)
            .map_err(|error| format!("failed to verify delta asset {asset}: {error}"))?;
    }
    let rebuilt = delta_reader
        .apply_to_base(&base)
        .map_err(|error| format!("failed to apply delta pack to previous zrpack: {error}"))?;
    let apply_verified = rebuilt.bytes == target_pack_bytes;
    if !apply_verified {
        return Err("delta pack apply verification did not reconstruct target zrpack".to_string());
    }
    Ok(Some(VerifiedDeltaWriteReport {
        manifest: delta_report.manifest,
        changed_assets: delta_report.changed_assets,
        removed_assets: delta_report.removed_assets,
        reused_assets: delta_report.reused_assets,
        apply_verified,
    }))
}

fn deterministic_double_run(
    enabled: bool,
    pack_assets: &[ZrPackInputAsset],
    first_bytes: &[u8],
    diagnostics: &mut Vec<String>,
) -> Result<bool, String> {
    if !enabled {
        return Ok(false);
    }
    let second = ZrPackWriter::write(pack_assets.to_vec())
        .map_err(|error| format!("failed to write deterministic comparison pack: {error}"))?;
    if second.bytes != first_bytes {
        diagnostics.push("deterministic pack double-run byte comparison failed".to_string());
        return Ok(false);
    }
    diagnostics.push("deterministic pack double-run byte comparison passed".to_string());
    Ok(true)
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct VerifiedDeltaWriteReport {
    manifest: ZrPackDeltaDocumentManifest,
    changed_assets: Vec<String>,
    removed_assets: Vec<String>,
    reused_assets: Vec<String>,
    apply_verified: bool,
}

#[cfg(test)]
mod tests {
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
        assert!(diagnostics
            .iter()
            .any(|diagnostic| diagnostic
                == "pack stage stopped because asset dependencies are missing"));

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
}
