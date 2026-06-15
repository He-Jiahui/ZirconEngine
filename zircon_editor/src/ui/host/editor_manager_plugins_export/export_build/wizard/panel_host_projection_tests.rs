use std::path::{Path, PathBuf};

use crate::ui::template_runtime::{
    EditorUiHostRuntime, RetainedUiHostNodeModel, RetainedUiHostValue,
};
use zircon_runtime_interface::ui::layout::UiSize;

use super::*;

#[test]
fn export_wizard_panel_retained_projection_applies_controls_and_slot_entries() {
    let mut runtime = EditorUiHostRuntime::default();
    register_export_wizard_panel_template(&mut runtime, desktop_export_panel_template_path())
        .expect("desktop export panel template should register");
    let plan = export_wizard_pipeline_plan(ready_export_options());
    let view_model = ExportWizardPanelViewModel::from_plan("export-retained-ready", &plan);

    let projection =
        export_wizard_panel_retained_projection(&runtime, &view_model, UiSize::new(960.0, 540.0))
            .expect("retained projection should build");

    assert_eq!(projection.document_id, EXPORT_WIZARD_TEMPLATE_DOCUMENT_ID);
    assert!(
        !projection
            .node_by_control_id(DESKTOP_EXPORT_START_BUTTON)
            .expect("start button should project")
            .disabled
    );
    assert!(
        projection
            .node_by_control_id(DESKTOP_EXPORT_CANCEL_BUTTON)
            .expect("cancel button should project")
            .disabled
    );

    let stage_anchor_id = projection
        .node_by_control_id(DESKTOP_EXPORT_STAGE_ROWS_SLOT)
        .expect("stage rows slot should project")
        .node_id
        .clone();
    let stage_rows = projection
        .nodes
        .iter()
        .filter(|node| node.parent_id.as_deref() == Some(stage_anchor_id.as_str()))
        .collect::<Vec<_>>();
    assert_eq!(stage_rows.len(), export_pipeline_stages().len());
    assert!(stage_rows.iter().any(|node| {
        node.text
            .as_deref()
            .is_some_and(|text| text.contains("Validate"))
    }));
    assert_eq!(
        string_property(stage_rows[0], "slot_kind"),
        Some("StageRows")
    );
    assert_eq!(string_property(stage_rows[0], "severity"), Some("Neutral"));
}

#[test]
fn export_wizard_panel_retained_projection_disables_start_for_missing_inputs() {
    let mut runtime = EditorUiHostRuntime::default();
    register_export_wizard_panel_template(&mut runtime, desktop_export_panel_template_path())
        .expect("desktop export panel template should register");
    let plan = export_wizard_pipeline_plan(ExportWizardPipelineOptions::new(
        "windows-release",
        "zircon-project.toml",
        "D:\\zircon-export",
    ));
    let view_model = ExportWizardPanelViewModel::from_plan("export-retained-missing", &plan);

    let projection =
        export_wizard_panel_retained_projection(&runtime, &view_model, UiSize::new(960.0, 540.0))
            .expect("retained projection should build");

    assert!(
        projection
            .node_by_control_id(DESKTOP_EXPORT_START_BUTTON)
            .expect("start button should project")
            .disabled
    );
    assert!(
        !projection
            .node_by_control_id(DESKTOP_EXPORT_GENERATE_PLAN_BUTTON)
            .expect("generate plan button should project")
            .disabled
    );

    let missing_anchor_id = projection
        .node_by_control_id(DESKTOP_EXPORT_MISSING_INPUTS_SLOT)
        .expect("missing inputs slot should project")
        .node_id
        .clone();
    let missing_rows = projection
        .nodes
        .iter()
        .filter(|node| node.parent_id.as_deref() == Some(missing_anchor_id.as_str()))
        .collect::<Vec<_>>();
    assert!(missing_rows.iter().any(|node| {
        node.text
            .as_deref()
            .is_some_and(|text| text.contains("source_asset_manifest"))
    }));
    assert!(missing_rows
        .iter()
        .all(|node| string_property(node, "slot_kind") == Some("MissingInputs")));
}

fn string_property<'a>(node: &'a RetainedUiHostNodeModel, key: &str) -> Option<&'a str> {
    match node.properties.get(key) {
        Some(RetainedUiHostValue::String(value)) => Some(value.as_str()),
        _ => None,
    }
}

fn ready_export_options() -> ExportWizardPipelineOptions {
    let mut options = ExportWizardPipelineOptions::new(
        "windows-release",
        "zircon-project.toml",
        "D:\\zircon-export",
    );
    options.source_asset_manifest = Some("D:\\zircon-export\\assets\\assets.json".to_string());
    options.host_executable = Some("D:\\zircon-export\\host\\zircon_game.exe".to_string());
    options.offline = true;
    options.dry_run = true;
    options
}

fn desktop_export_panel_template_path() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("editor crate should be inside repo root")
        .join("zircon_plugins")
        .join("editor_build_export_desktop")
        .join("editor")
        .join("panel.v2.ui.toml")
}
