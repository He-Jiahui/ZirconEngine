use crate::core::framework::platform::RuntimeTargetMode;
use crate::core::framework::project::ProjectPluginManifest;

pub(in crate::plugin::export_build_plan) fn project_duplicate_selection_diagnostics(
    manifest: &ProjectPluginManifest,
    target: RuntimeTargetMode,
) -> (Vec<String>, Vec<String>) {
    let mut diagnostics = Vec::new();
    let mut fatal_diagnostics = Vec::new();
    let mut seen_plugins = Vec::new();
    for selection in manifest
        .selections
        .iter()
        .filter(|selection| selection.enabled && selection.supports_target(target))
    {
        if let Some((_, first_required)) = seen_plugins
            .iter()
            .find(|(plugin_id, _)| plugin_id == &selection.id)
        {
            let diagnostic = format!(
                "project plugin selection id `{}` is declared more than once",
                selection.id
            );
            if selection.required || *first_required {
                fatal_diagnostics.push(diagnostic.clone());
            }
            diagnostics.push(diagnostic);
        } else {
            seen_plugins.push((selection.id.clone(), selection.required));
        }

        let mut seen_features = Vec::new();
        for feature in selection
            .features
            .iter()
            .filter(|feature| feature.enabled && feature.supports_target(target))
        {
            if let Some((_, first_required)) = seen_features
                .iter()
                .find(|(feature_id, _)| feature_id == &feature.id)
            {
                let diagnostic = format!(
                    "project plugin feature id `{}` is declared more than once under project plugin `{}`",
                    feature.id, selection.id
                );
                if feature.required || *first_required {
                    fatal_diagnostics.push(diagnostic.clone());
                }
                diagnostics.push(diagnostic);
            } else {
                seen_features.push((feature.id.clone(), feature.required));
            }
        }
    }
    (diagnostics, fatal_diagnostics)
}
