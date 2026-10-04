use serde::Serialize;
use std::ffi::OsString;
use std::fs;
use std::path::Path;
use std::process::ExitCode;
use zircon_runtime_interface::export::ExportStage;

use super::args::{parse, usage};
use super::error::{ExportPackError, ExportPackResult};
use super::manifest::{ExportAssetPackManifest, ExportPackInputSource};
use crate::pack::{
    ZrPackDeltaDocumentManifest, ZrPackDeltaReader, ZrPackDeltaWriter, ZrPackDocumentManifest,
    ZrPackReader, ZrPackTrimReport, ZrPackWriter,
};

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct ExportPackReport {
    pub stage: ExportStage,
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

pub fn run(args: impl IntoIterator<Item = OsString>) -> ExportPackResult<ExitCode> {
    let Some(args) = parse(args)? else {
        println!("{}", usage("zircon export pack writer"));
        return Ok(ExitCode::SUCCESS);
    };

    let manifest_dir = args.manifest.parent().unwrap_or_else(|| Path::new("."));
    let asset_manifest_text = fs::read_to_string(&args.manifest).map_err(|source| {
        ExportPackError::ReadAssetManifest {
            path: args.manifest.clone(),
            source,
        }
    })?;
    let asset_manifest = serde_json::from_str::<ExportAssetPackManifest>(&asset_manifest_text)
        .map_err(|source| ExportPackError::DecodeAssetManifest { source })?;
    let pack_inputs = asset_manifest.pack_inputs(manifest_dir);
    let mut diagnostics = pack_inputs.diagnostics.clone();
    let fatal_preflight = pack_inputs.trim_report.has_missing_dependencies()
        || pack_inputs.trim_report.has_duplicate_assets()
        || !pack_inputs.asset_source_errors.is_empty();

    let mut report = if fatal_preflight {
        ExportPackReport {
            stage: ExportStage::Pack,
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
        match ZrPackWriter::write_files(
            pack_inputs
                .pack_sources
                .iter()
                .map(|source| (source.path.as_str(), source.source.as_path())),
        ) {
            Ok(write_report) => {
                let deterministic_double_run = deterministic_double_run(
                    args.determinism_check,
                    &pack_inputs.pack_sources,
                    &write_report.bytes,
                    &mut diagnostics,
                )?;
                if let Some(parent) = args.pack.parent() {
                    if !parent.as_os_str().is_empty() {
                        fs::create_dir_all(parent).map_err(|source| {
                            ExportPackError::CreatePackDirectory {
                                path: parent.to_path_buf(),
                                source,
                            }
                        })?;
                    }
                }
                fs::write(&args.pack, &write_report.bytes).map_err(|source| {
                    ExportPackError::WritePack {
                        path: args.pack.clone(),
                        source,
                    }
                })?;
                let delta_report = write_delta_pack_if_requested(&args, &write_report.bytes)?;
                ExportPackReport {
                    stage: ExportStage::Pack,
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
            Err(error) => {
                diagnostics.push(format!("failed to write zrpack: {error}"));
                ExportPackReport {
                    stage: ExportStage::Pack,
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
            }
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
    .map_err(|source| ExportPackError::EncodeReport { source })?;

    if let Some(report_path) = &args.report {
        if let Some(parent) = report_path.parent() {
            if !parent.as_os_str().is_empty() {
                fs::create_dir_all(parent).map_err(|source| {
                    ExportPackError::CreateReportDirectory {
                        path: parent.to_path_buf(),
                        source,
                    }
                })?;
            }
        }
        fs::write(report_path, &json).map_err(|source| ExportPackError::WriteReport {
            path: report_path.clone(),
            source,
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
) -> ExportPackResult<Option<VerifiedDeltaWriteReport>> {
    let Some(previous_pack) = &args.previous_pack else {
        return Ok(None);
    };
    let Some(delta_pack) = &args.delta_pack else {
        return Ok(None);
    };
    let previous_bytes =
        fs::read(previous_pack).map_err(|source| ExportPackError::ReadPreviousPack {
            path: previous_pack.clone(),
            source,
        })?;
    let base = ZrPackReader::from_bytes(previous_bytes)
        .map_err(|source| ExportPackError::ReadPreviousZrPack { source })?;
    let target = ZrPackReader::from_bytes(target_pack_bytes.to_vec())
        .map_err(|source| ExportPackError::ReadNewlyWrittenZrPack { source })?;
    let delta_report = ZrPackDeltaWriter::write(&base, &target)
        .map_err(|source| ExportPackError::WriteDeltaZrPack { source })?;
    if let Some(parent) = delta_pack.parent() {
        if !parent.as_os_str().is_empty() {
            fs::create_dir_all(parent).map_err(|source| {
                ExportPackError::CreateDeltaPackDirectory {
                    path: parent.to_path_buf(),
                    source,
                }
            })?;
        }
    }
    fs::write(delta_pack, &delta_report.bytes).map_err(|source| {
        ExportPackError::WriteDeltaPack {
            path: delta_pack.clone(),
            source,
        }
    })?;
    let delta_reader = ZrPackDeltaReader::from_bytes(delta_report.bytes)
        .map_err(|source| ExportPackError::VerifyWrittenDeltaZrPack { source })?;
    if let Some(asset) = delta_report.changed_assets.first() {
        let _ = delta_reader.read_changed_asset(asset).map_err(|source| {
            ExportPackError::VerifyDeltaAsset {
                asset: asset.clone(),
                source,
            }
        })?;
    }
    let rebuilt = delta_reader
        .apply_to_base(&base)
        .map_err(|source| ExportPackError::ApplyDeltaPack { source })?;
    let apply_verified = rebuilt.bytes == target_pack_bytes;
    if !apply_verified {
        return Err(ExportPackError::DeltaApplyVerificationMismatch);
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
    pack_sources: &[ExportPackInputSource],
    first_bytes: &[u8],
    diagnostics: &mut Vec<String>,
) -> ExportPackResult<bool> {
    if !enabled {
        return Ok(false);
    }
    let second = match ZrPackWriter::write_files(
        pack_sources
            .iter()
            .map(|source| (source.path.as_str(), source.source.as_path())),
    ) {
        Ok(report) => report,
        Err(error) => {
            diagnostics.push(format!(
                "deterministic pack comparison source read failed: {error}"
            ));
            return Ok(false);
        }
    };
    if second.bytes != first_bytes {
        diagnostics.push("deterministic pack double-run byte comparison failed".to_string());
        return Ok(false);
    }
    diagnostics.push("deterministic pack double-run byte comparison passed".to_string());
    Ok(true)
}

#[cfg(test)]
#[path = "run/tests/optimization_tests.rs"]
mod optimization_tests;

#[derive(Clone, Debug, PartialEq, Eq)]
struct VerifiedDeltaWriteReport {
    manifest: ZrPackDeltaDocumentManifest,
    changed_assets: Vec<String>,
    removed_assets: Vec<String>,
    reused_assets: Vec<String>,
    apply_verified: bool,
}

#[cfg(test)]
#[path = "tests/run.rs"]
mod tests;
