//! 按清单原顺序报告重复插件及特征；required 首行或重复行使冲突升级为阻断诊断。
use crate::core::framework::project::ProjectPluginManifest;

use super::ProjectPluginManifestValidationProjection;

/// 用 projection 中首次 required 分类保持诊断稳定；调用方应在清理重复行前从原清单取证。
pub(in crate::plugin::export_build_plan) fn project_duplicate_selection_diagnostics(
    manifest: &ProjectPluginManifest,
    projection: &ProjectPluginManifestValidationProjection,
) -> (Vec<String>, Vec<String>) {
    let mut diagnostics = Vec::with_capacity(manifest.selections.len());
    let mut fatal_diagnostics = Vec::with_capacity(manifest.selections.len());
    for (selection_index, selection) in manifest.selections.iter().enumerate() {
        if let Some(first_required) = projection.duplicate_selection_first_required(selection_index)
        {
            let diagnostic = format!(
                "project plugin selection id `{}` is declared more than once",
                selection.id
            );
            if selection.required || first_required {
                fatal_diagnostics.push(diagnostic.clone());
            }
            diagnostics.push(diagnostic);
        }

        for (feature_index, feature) in selection.features.iter().enumerate() {
            if let Some(first_required) =
                projection.duplicate_feature_first_required(selection_index, feature_index)
            {
                let diagnostic = format!(
                    "project plugin feature id `{}` is declared more than once under project plugin `{}`",
                    feature.id, selection.id
                );
                if feature.required || first_required {
                    fatal_diagnostics.push(diagnostic.clone());
                }
                diagnostics.push(diagnostic);
            }
        }
    }
    (diagnostics, fatal_diagnostics)
}

#[cfg(test)]
#[path = "tests/duplicates_optimization_batch_20260830bt_runtime_tests.rs"]
mod optimization_batch_20260830bt_runtime_tests;
