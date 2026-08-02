//! Extension and asset-type materialization for a catalog generation.

use crate::core::asset::{AssetTypeContribution, AssetTypeRegistry};
use crate::core::editor_extension::{EditorExtensionRegistry, EditorExtensionRegistryError};

use super::extension_catalog_report::EditorExtensionCatalogReport;
use super::registration::EditorPluginRegistrationReport;

pub(super) fn build_editor_extensions<'a>(
    catalog_generation: u64,
    registrations: impl IntoIterator<Item = &'a EditorPluginRegistrationReport>,
) -> EditorExtensionCatalogReport {
    let mut registry = EditorExtensionRegistry::default();
    let mut diagnostics = Vec::<(usize, String)>::new();
    let mut diagnostic_sequence = 0;
    let builtin_sequence = take_diagnostic_sequence(&mut diagnostic_sequence);
    let mut asset_types = match AssetTypeRegistry::with_builtins() {
        Ok(registry) => registry,
        Err(error) => {
            diagnostics.push((builtin_sequence, error.to_string()));
            AssetTypeRegistry::default()
        }
    };
    let mut asset_type_contributions = Vec::<(String, AssetTypeContribution)>::new();
    let mut asset_type_contribution_sequences = Vec::new();
    for registration in registrations {
        for view in registration.extensions.views() {
            push_editor_extension_result(
                registry.register_view((*view).clone()),
                &mut diagnostics,
                &mut diagnostic_sequence,
            );
        }
        for drawer in registration.extensions.drawers() {
            push_editor_extension_result(
                registry.register_drawer((*drawer).clone()),
                &mut diagnostics,
                &mut diagnostic_sequence,
            );
        }
        for menu_item in registration.extensions.menu_items() {
            push_editor_extension_result(
                registry.register_menu_item((*menu_item).clone()),
                &mut diagnostics,
                &mut diagnostic_sequence,
            );
        }
        for customization in registration.extensions.inspector_customizations() {
            push_editor_extension_result(
                registry.register_inspector_customization((*customization).clone()),
                &mut diagnostics,
                &mut diagnostic_sequence,
            );
        }
        for ui_template in registration.extensions.ui_templates() {
            push_editor_extension_result(
                registry.register_ui_template((*ui_template).clone()),
                &mut diagnostics,
                &mut diagnostic_sequence,
            );
        }
        for asset_importer in registration.extensions.asset_importers() {
            push_editor_extension_result(
                registry.register_asset_importer((*asset_importer).clone()),
                &mut diagnostics,
                &mut diagnostic_sequence,
            );
        }
        for contribution in registration.extensions.asset_type_contributions() {
            asset_type_contribution_sequences
                .push(take_diagnostic_sequence(&mut diagnostic_sequence));
            asset_type_contributions.push((
                registration.package_manifest.id.clone(),
                (*contribution).clone(),
            ));
        }
        for scene_mode in registration.extensions.scene_mode_registrations() {
            push_editor_extension_result(
                registry.register_scene_mode((*scene_mode).clone()),
                &mut diagnostics,
                &mut diagnostic_sequence,
            );
        }
        for graph_editor in registration.extensions.graph_editors() {
            push_editor_extension_result(
                registry.register_graph_editor((*graph_editor).clone()),
                &mut diagnostics,
                &mut diagnostic_sequence,
            );
        }
        for palette in registration.extensions.graph_node_palettes() {
            push_editor_extension_result(
                registry.register_graph_node_palette((*palette).clone()),
                &mut diagnostics,
                &mut diagnostic_sequence,
            );
        }
        for timeline_editor in registration.extensions.timeline_editors() {
            push_editor_extension_result(
                registry.register_timeline_editor((*timeline_editor).clone()),
                &mut diagnostics,
                &mut diagnostic_sequence,
            );
        }
        for track_type in registration.extensions.timeline_track_types() {
            push_editor_extension_result(
                registry.register_timeline_track_type((*track_type).clone()),
                &mut diagnostics,
                &mut diagnostic_sequence,
            );
        }
        for operation in registration.extensions.pending_commands().cloned() {
            push_editor_extension_result(
                registry.register_command(operation),
                &mut diagnostics,
                &mut diagnostic_sequence,
            );
        }
    }
    let asset_type_report = asset_types.apply_contributions(asset_type_contributions);
    for (input_index, error) in asset_type_report.into_errors() {
        diagnostics.push((
            asset_type_contribution_sequences[input_index],
            error.to_string(),
        ));
    }
    diagnostics.sort_by_key(|(sequence, _)| *sequence);
    let diagnostics = diagnostics
        .into_iter()
        .map(|(_, diagnostic)| diagnostic)
        .collect();
    EditorExtensionCatalogReport {
        catalog_generation,
        active_manager_generation: None,
        registry,
        asset_types,
        diagnostics,
    }
}

fn push_editor_extension_result(
    result: Result<(), EditorExtensionRegistryError>,
    diagnostics: &mut Vec<(usize, String)>,
    diagnostic_sequence: &mut usize,
) {
    let sequence = take_diagnostic_sequence(diagnostic_sequence);
    if let Err(error) = result {
        diagnostics.push((sequence, error.to_string()));
    }
}

fn take_diagnostic_sequence(next_sequence: &mut usize) -> usize {
    let sequence = *next_sequence;
    *next_sequence = sequence.saturating_add(1);
    sequence
}
