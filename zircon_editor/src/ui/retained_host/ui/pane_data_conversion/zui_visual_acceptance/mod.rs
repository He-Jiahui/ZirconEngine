//! Catalog-driven captures through the product retained host. Capture alone is
//! deliberately insufficient to publish a layout acceptance result.

mod capture;
mod component_input;
mod contract;
mod geometry;
mod painter_receipts;
mod product_presentation;
mod readiness;
mod source;
mod source_identity;
mod state;
#[cfg(test)]
#[path = "tests/cases.rs"]
mod tests;
mod theme;

use std::path::{Path, PathBuf};

use contract::{contained, hash_file, read_json, write_json, Catalog, ReviewCase};
use serde_json::json;
use zircon_runtime::core::CoreHandle;
use zircon_runtime_interface::runtime_build_set::ZrRuntimeBuildSetId;

#[derive(Debug)]
pub(crate) struct CaptureSummary {
    pub(crate) captured: usize,
    pub(crate) failed: usize,
    pub(crate) pending: usize,
    pub(crate) report_path: PathBuf,
}

pub(crate) fn export_product_workbench_case_snapshots(
    repo_root: &Path,
    output_path: &Path,
) -> Result<(), String> {
    product_presentation::export_product_workbench_case_snapshots(repo_root, output_path)
}

pub(crate) fn export_product_workbench_case_snapshots_with_context(
    repo_root: &Path,
    output_path: &Path,
    core: &CoreHandle,
    build_set: &ZrRuntimeBuildSetId,
) -> Result<(), String> {
    product_presentation::export_product_workbench_case_snapshots_with_context(
        repo_root,
        output_path,
        core,
        build_set,
    )
}

impl CaptureSummary {
    #[cfg(test)]
    fn is_ready(&self) -> bool {
        self.captured > 0 && self.failed == 0 && self.pending == 0
    }
}

#[cfg(test)]
include!("tests/visual_capture_cases.rs");

pub(crate) fn capture_catalog() -> Result<CaptureSummary, String> {
    let repo = std::env::var_os("ZUI_LAYOUT_REPO_ROOT")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(".."))
        .canonicalize()
        .map_err(|error| error.to_string())?;
    capture_catalog_at(&repo, None)
}

pub(crate) fn capture_catalog_with_context(
    repo_root: &Path,
    core: &CoreHandle,
    build_set: &ZrRuntimeBuildSetId,
) -> Result<CaptureSummary, String> {
    let repo = repo_root
        .canonicalize()
        .map_err(|error| error.to_string())?;
    capture_catalog_at(&repo, Some((core, build_set)))
}

fn capture_catalog_at(
    repo: &Path,
    context: Option<(&CoreHandle, &ZrRuntimeBuildSetId)>,
) -> Result<CaptureSummary, String> {
    let output = repo.join("docs/layout");
    let catalog: Catalog = serde_json::from_value(read_json(&output.join("catalog.json"))?)
        .map_err(|error| error.to_string())?;
    let source_filter = std::env::var("ZUI_LAYOUT_SOURCE").ok();
    let case_filter = std::env::var("ZUI_LAYOUT_CASE").ok();
    let executable = std::env::current_exe().map_err(|error| error.to_string())?;
    let renderer_sha = hash_file(&executable)?;
    let mut evidence = Vec::new();
    for entry in catalog.entries {
        if !(entry.source_path.starts_with("zircon_editor/")
            || entry.source_path.starts_with("zircon_plugins/"))
            || source_filter
                .as_ref()
                .is_some_and(|filter| !entry.source_path.contains(filter))
        {
            continue;
        }
        for raw_case in &entry.cases {
            let case: ReviewCase =
                serde_json::from_value(raw_case.clone()).map_err(|error| error.to_string())?;
            if case.host == "fixture" {
                continue;
            }
            if case_filter
                .as_ref()
                .is_some_and(|filter| &case.id != filter)
            {
                continue;
            }
            let relative = format!("{}/{}/evidence", entry.category, entry.name);
            let directory = contained(&output, &relative)?;
            std::fs::create_dir_all(&directory).map_err(|error| error.to_string())?;
            let mut record = json!({
                "sourcePath": entry.source_path, "caseId": case.id,
                "sourceSha256": entry.source_sha256,
                "inputSha256": entry.source_sha256,
                "dependencySha256": entry.dependency_sha256,
                "caseSha256": contract::canonical_hash(raw_case)?,
                "rendererKind": "zircon-editor-retained-host",
                "rendererPath": executable.to_string_lossy(),
                "rendererSha256": renderer_sha,
                "screenshotPath": "", "screenshotSha256": "", "status": "failed",
                "runtimeAssetFingerprints": [],
                "dependencyFingerprints": entry.dependency_fingerprints,
            });
            if let Some(host) = &case.review_host {
                record["reviewHostSha256"] = json!(host.sha256);
            }
            if let Some((_, build_set)) = context {
                record["runtimeBuildSetId"] = json!(build_set);
            }
            let result = (|| {
                entry.verify(&repo, &output)?;
                case.validate(&entry.source_path)?;
                let captured = match context {
                    Some((core, build_set)) => capture::render_with_context(
                        repo, &entry, raw_case, &case, &directory, core, build_set,
                    )?,
                    None => capture::render(repo, &entry, raw_case, &case, &directory)?,
                };
                entry.verify(&repo, &output)?;
                contract::verify_review_host(&output, &entry, &case)?;
                Ok::<_, String>(captured)
            })();
            match result {
                Ok(captured) => {
                    for (key, value) in captured {
                        record[key] = value;
                    }
                    record["status"] = record["nativeReadiness"]["status"].clone();
                    if record["status"] == "pending" {
                        record["pendingReason"] = json!(record["nativeReadiness"]["reasons"]
                            .as_array()
                            .into_iter()
                            .flatten()
                            .filter_map(serde_json::Value::as_str)
                            .collect::<Vec<_>>()
                            .join("; "));
                    }
                }
                Err(error) => record["error"] = json!(error),
            }
            let detail = if record["status"] == "pending" {
                &record["pendingReason"]
            } else {
                &record["error"]
            };
            println!(
                "{} {} [{}]: {}",
                entry.source_path, case.id, record["status"], detail
            );
            evidence.push(record);
        }
    }
    if evidence.is_empty() {
        return Err("no Editor catalog cases matched the requested filters".into());
    }
    let report = output.join("editor-native-evidence.json");
    write_json(
        &report,
        &json!({
            "schema": "dev.zircon.zui.native-evidence", "version": 1,
            "rendererSha256": renderer_sha,
            "runtimeBuildSetId": context.map(|(_, build_set)| build_set),
            "evidence": evidence,
        }),
    )?;
    let failed = evidence
        .iter()
        .filter(|item| item["status"] == "failed")
        .count();
    let pending = evidence
        .iter()
        .filter(|item| item["status"] == "pending")
        .count();
    Ok(CaptureSummary {
        captured: evidence.len(),
        failed,
        pending,
        report_path: report,
    })
}
