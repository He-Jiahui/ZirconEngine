use crate::asset::project::ProjectManifest;
use crate::{plugin::ExportPackagingStrategy, plugin::ExportProfile, plugin::ProjectPluginSelection};

use super::asset_manifest_template::asset_manifest_template;
use super::cargo_manifest_template::cargo_manifest_template;
use super::main_template::main_template;
use super::native_plugin_load_manifest_template::native_plugin_load_manifest_template;
use super::plugin_selection_template::plugin_selection_template;
use super::{ExportGeneratedFile, ExportLinkedRuntimeCrate};

pub(super) fn generated_files_for_profile(
    manifest: &ProjectManifest,
    profile: &ExportProfile,
    project_plugin_selections: &[&ProjectPluginSelection],
    linked_runtime_crates: &[ExportLinkedRuntimeCrate],
    native_dynamic_packages: &[String],
) -> Vec<ExportGeneratedFile> {
    let mut files = Vec::new();
    if !native_dynamic_packages.is_empty() {
        files.push(ExportGeneratedFile {
            path: "plugins/native_plugins.toml".to_string(),
            purpose: "native dynamic plugin loading manifest".to_string(),
            contents: native_plugin_load_manifest_template(native_dynamic_packages),
        });
    }

    if !source_template_enabled(&profile.strategies) {
        return files;
    }

    files.extend([
        ExportGeneratedFile {
            path: "Cargo.toml".to_string(),
            purpose: "generated runtime package manifest".to_string(),
            contents: cargo_manifest_template(profile, linked_runtime_crates),
        },
        ExportGeneratedFile {
            path: "src/main.rs".to_string(),
            purpose: "generated platform runtime entry point".to_string(),
            contents: main_template(profile, !native_dynamic_packages.is_empty()),
        },
        ExportGeneratedFile {
            path: "src/zircon_plugins.rs".to_string(),
            purpose: "generated plugin selection code".to_string(),
            contents: plugin_selection_template(
                profile,
                project_plugin_selections,
                linked_runtime_crates,
            ),
        },
        ExportGeneratedFile {
            path: "assets/zircon-project.toml".to_string(),
            purpose: "project runtime manifest copy".to_string(),
            contents: asset_manifest_template(manifest),
        },
    ]);
    files
}

pub(super) fn source_template_enabled(strategies: &[ExportPackagingStrategy]) -> bool {
    strategies.contains(&ExportPackagingStrategy::SourceTemplate)
}
