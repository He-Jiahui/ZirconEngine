use crate::core::framework::project::ProjectPluginManifest;

use super::ProjectPluginManifestValidationProjection;

pub(in crate::plugin::export_build_plan) fn project_duplicate_selection_diagnostics(
    manifest: &ProjectPluginManifest,
    projection: &ProjectPluginManifestValidationProjection,
) -> (Vec<String>, Vec<String>) {
    let mut diagnostics = Vec::new();
    let mut fatal_diagnostics = Vec::new();
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
