use super::summary::ZuiVisualEvidenceSummary;
use zircon_runtime::core::CoreHandle;
use zircon_runtime_interface::runtime_build_set::ZrRuntimeBuildSetId;

/// Write product painter evidence without changing catalog acceptance decisions.
pub fn export_zui_visual_evidence() -> Result<ZuiVisualEvidenceSummary, String> {
    let summary = super::super::ui::capture_zui_visual_evidence()?;
    Ok(ZuiVisualEvidenceSummary {
        captured: summary.captured,
        failed: summary.failed,
        pending: summary.pending,
        report_path: summary.report_path,
    })
}

/// Write the actual product states consumed by both workbench renderers.
pub fn export_zui_workbench_product_snapshots(
    repo_root: &std::path::Path,
    output_path: &std::path::Path,
) -> Result<(), String> {
    super::super::ui::export_zui_workbench_product_snapshots(repo_root, output_path)
}

/// Capture through the live App composition and its preflighted Runtime BuildSet.
pub fn export_zui_visual_evidence_with_context(
    repo_root: &std::path::Path,
    core: &CoreHandle,
    build_set: &ZrRuntimeBuildSetId,
) -> Result<ZuiVisualEvidenceSummary, String> {
    let summary =
        super::super::ui::capture_zui_visual_evidence_with_context(repo_root, core, build_set)?;
    Ok(ZuiVisualEvidenceSummary {
        captured: summary.captured,
        failed: summary.failed,
        pending: summary.pending,
        report_path: summary.report_path,
    })
}

/// Export product states using the same Core and Runtime identity as the normal Editor.
pub fn export_zui_workbench_product_snapshots_with_context(
    repo_root: &std::path::Path,
    output_path: &std::path::Path,
    core: &CoreHandle,
    build_set: &ZrRuntimeBuildSetId,
) -> Result<(), String> {
    super::super::ui::export_zui_workbench_product_snapshots_with_context(
        repo_root,
        output_path,
        core,
        build_set,
    )
}
