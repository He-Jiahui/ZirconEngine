use std::error::Error;

use zircon_runtime::asset::ProjectInfo;
use zircon_runtime::core::CoreHandle;
use zircon_runtime::scene::NodeRecord;
use zircon_runtime_interface::ui::binding::UiBindingValue;

use crate::core::editor_event::{EditorEventRecord, EditorEventSource};
use crate::core::gateway::SharedEditorRuntimeGateway;
use crate::ui::binding::{EditorUiBinding, EditorUiBindingPayload, SelectionCommand};
use crate::ui::retained_host::PaneSurfaceHostContext;
use crate::ui::workbench::snapshot::EditorDataSnapshot;

use super::{
    apply_host_appearance_from_tokens, install_editor_v2_design_tokens, wire_callbacks,
    EditorHostRunConfig, RetainedEditorHost, UiHostWindow,
};

const POSITION_X_CONTROL: &str = "WorkbenchTransformPositionX";
const POSITION_X_BINDING: &str = "Inspector/TransformPositionXCommit";
const POSITION_X_FIELD: &str = "transform.translation.x";
const SCALE_X_CONTROL: &str = "WorkbenchTransformScaleX";
const SCALE_X_BINDING: &str = "Inspector/TransformScaleXCommit";
const SCALE_X_FIELD: &str = "transform.scale.x";
const SAVE_PROJECT_CONTROL: &str = "SaveProject";
const SAVE_PROJECT_ACTION: &str = "WorkbenchMenuBar/SaveProject";
const HISTORY_CONTROL: &str = "WorkbenchMenuBar";
const HISTORY_UNDO_ACTION: &str = "workbench.history.undo";
const HISTORY_REDO_ACTION: &str = "workbench.history.redo";
const HIERARCHY_SELECT_CLI_BINDING: &str = "Hierarchy/SelectCube:onClick";
const POSITION_X_CLI_BINDING: &str = "Inspector/TransformPositionXCommit:onSubmit";
const SCALE_X_CLI_BINDING: &str = "Inspector/TransformScaleXCommit:onSubmit";
const HISTORY_UNDO_CLI_BINDING: &str = "WorkbenchMenuBar/Undo:onClick";
const HISTORY_REDO_CLI_BINDING: &str = "WorkbenchMenuBar/Redo:onClick";
const SAVE_PROJECT_CLI_BINDING: &str = "WorkbenchMenuBar/SaveProject:onClick";

/// The authoritative retained-host state captured after a callback-driven automation sequence.
pub struct RetainedHostAutomationResult {
    pub records: Vec<EditorEventRecord>,
    pub editor_snapshot: EditorDataSnapshot,
    pub scene_nodes: Vec<NodeRecord>,
    pub project_info: ProjectInfo,
    pub opened_project_inspection_generation: u64,
}

/// Builds the production retained host, invokes its wired callback surface, and exits without
/// entering the native event loop. This keeps automation on the same template and callback path
/// as an interactive editor session.
pub fn run_retained_host_automation(
    core: CoreHandle,
    runtime_gateway: SharedEditorRuntimeGateway,
    config: EditorHostRunConfig,
    bindings: &[EditorUiBinding],
) -> Result<RetainedHostAutomationResult, Box<dyn Error>> {
    let (startup_request, prepared_project, _, editor_plugin_registrations, hub_handshake) =
        config.into_parts();
    reject_automation_hub_handshake(hub_handshake)?;
    let ui = UiHostWindow::new()?;
    let mut retained_host = RetainedEditorHost::new(
        core,
        runtime_gateway,
        ui.clone_strong(),
        startup_request,
        prepared_project,
        None,
    )?;
    let settings_snapshot = retained_host.editor_manager.context().settings().snapshot();
    apply_host_appearance_from_tokens(settings_snapshot.design_tokens());
    install_editor_v2_design_tokens(settings_snapshot.as_ref());
    for registration in editor_plugin_registrations {
        retained_host
            .runtime
            .register_editor_plugin_registration(registration)?;
    }
    retained_host.sync_plugin_template_documents_if_changed()?;

    let host = std::rc::Rc::new(std::cell::RefCell::new(retained_host));
    wire_callbacks(&ui, &host);
    host.borrow_mut().self_handle = Some(std::rc::Rc::downgrade(&host));
    host.borrow_mut().refresh_ui();

    let result: Result<RetainedHostAutomationResult, Box<dyn Error>> =
        if let Some(error) = ui.take_fatal_failure() {
            Err(format!("retained-host automation initialization failed: {error}").into())
        } else {
            invoke_automation_callbacks(&ui, &host, bindings)
        };
    let host_window_diagnostics = ui.take_host_diagnostics();
    let settings_persistence = host
        .borrow()
        .editor_manager
        .context()
        .settings_persistence()
        .clone();
    {
        let retained_host = host.borrow();
        super::emit_host_window_diagnostics(
            retained_host.runtime.context().logs(),
            host_window_diagnostics,
        );
    }
    let project_close_result = {
        let retained_host = host.borrow();
        retained_host.editor_manager.close_project()
    };
    drop(host);
    let settings_shutdown = match settings_persistence.flush_then_shutdown() {
        Ok(shutdown) => shutdown,
        Err(error) => {
            let guard = settings_persistence.shutdown();
            drop(guard);
            return Err(error.into());
        }
    };
    settings_shutdown.finish()?;
    project_close_result?;
    result
}

fn reject_automation_hub_handshake(
    hub_handshake: Option<super::HubEditorHandshake>,
) -> Result<(), std::io::Error> {
    if hub_handshake.is_some() {
        return Err(std::io::Error::other(
            "retained-host automation cannot acknowledge a Hub editor handshake",
        ));
    }
    Ok(())
}

fn invoke_automation_callbacks(
    ui: &UiHostWindow,
    host: &std::rc::Rc<std::cell::RefCell<RetainedEditorHost>>,
    bindings: &[EditorUiBinding],
) -> Result<RetainedHostAutomationResult, Box<dyn Error>> {
    let pane_surface_host = ui.global::<PaneSurfaceHostContext>();
    let mut records = Vec::new();
    for (index, binding) in bindings.iter().enumerate() {
        let journal_start = host.borrow().runtime.journal().records().len();
        invoke_supported_binding(&pane_surface_host, host, binding).map_err(|error| {
            format!(
                "retained-host automation binding {index} ('{}') failed: {error}",
                binding.native_binding()
            )
        })?;
        host.borrow_mut().refresh_ui();
        if let Some(error) = ui.take_fatal_failure() {
            return Err(format!(
                "retained-host automation binding {index} ('{}') triggered a host failure: {error}",
                binding.native_binding()
            )
            .into());
        }
        let journal = host.borrow();
        let action_journal = journal.runtime.journal();
        let action_records = &action_journal.records()[journal_start..];
        if action_records.is_empty() {
            return Err(format!(
                "retained-host automation binding {index} ('{}') completed without a callback journal record",
                binding.native_binding()
            )
            .into());
        }
        records.extend(normalize_cli_action_records(
            index,
            binding,
            action_records,
        )?);
    }

    let host = host.borrow();
    let editor_snapshot = host.runtime.editor_snapshot();
    let project_info = host
        .startup_session
        .project
        .as_ref()
        .ok_or_else(|| {
            "retained-host automation completed without an authoritative project".to_string()
        })?
        .project_info
        .clone();
    let project_scene = host.runtime.project_scene_snapshot().ok_or_else(|| {
        "retained-host automation completed without an authoritative scene".to_string()
    })?;
    let opened_project_inspection_generation = project_scene.inspection_artifact().generation();
    let scene_nodes = editor_snapshot
        .scene_entries
        .iter()
        .map(|entry| {
            project_scene.node_record(entry.entity).ok_or_else(|| {
                format!(
                    "retained-host automation could not snapshot scene node {} ('{}')",
                    entry.entity, entry.display_name
                )
            })
        })
        .collect::<Result<Vec<_>, _>>()?;

    Ok(RetainedHostAutomationResult {
        records,
        editor_snapshot,
        scene_nodes,
        project_info,
        opened_project_inspection_generation,
    })
}

#[cfg(test)]
mod hub_handshake_tests {
    use std::str::FromStr;

    use zircon_runtime_interface::hub_protocol::HubSessionToken;

    use super::reject_automation_hub_handshake;

    #[test]
    fn automation_rejects_a_hub_handshake_instead_of_dropping_its_terminal_outcome() {
        let handshake = super::super::HubEditorHandshake::new(
            "E:/Projects/Automation",
            HubSessionToken::from_str("0d9a5890-0e44-4e2a-b77e-3e5d4fdf1e52")
                .expect("valid Hub session token"),
        );

        let error = reject_automation_hub_handshake(Some(handshake))
            .expect_err("automation cannot report the interactive Hub ready outcome");

        assert!(error.to_string().contains("cannot acknowledge"));
    }

    #[test]
    fn automation_allows_a_regular_non_hub_config() {
        assert!(reject_automation_hub_handshake(None).is_ok());
    }
}

pub(super) fn normalize_cli_action_records(
    index: usize,
    binding: &EditorUiBinding,
    action_records: &[EditorEventRecord],
) -> Result<Vec<EditorEventRecord>, String> {
    let binding_path = canonical_cli_binding_path(binding)?;
    if let Some(record) = action_records
        .iter()
        .find(|record| record.result.error.is_some())
    {
        return Err(format!(
            "retained-host automation binding {index} ('{binding_path}') recorded an editor callback failure at sequence {}; see retained-host diagnostics",
            record.sequence.0
        ));
    }

    Ok(action_records
        .iter()
        .cloned()
        .map(|mut record| {
            record.source = EditorEventSource::Cli;
            record.binding_path = Some(binding_path.to_string());
            record
        })
        .collect())
}

pub(super) fn canonical_cli_binding_path(
    binding: &EditorUiBinding,
) -> Result<&'static str, String> {
    match binding.payload() {
        EditorUiBindingPayload::SelectionCommand(SelectionCommand::SelectSceneNode { .. }) => {
            Ok(HIERARCHY_SELECT_CLI_BINDING)
        }
        EditorUiBindingPayload::InspectorFieldBatch { changes, .. } => {
            let (_, binding_id, _) = transform_x_commit(changes)?;
            match binding_id {
                POSITION_X_BINDING => Ok(POSITION_X_CLI_BINDING),
                SCALE_X_BINDING => Ok(SCALE_X_CLI_BINDING),
                _ => unreachable!("transform X automation maps only known Inspector bindings"),
            }
        }
        EditorUiBindingPayload::MenuAction { action_id }
            if action_id == "workbench.project.save" =>
        {
            Ok(SAVE_PROJECT_CLI_BINDING)
        }
        EditorUiBindingPayload::MenuAction { action_id } if action_id == HISTORY_UNDO_ACTION => {
            Ok(HISTORY_UNDO_CLI_BINDING)
        }
        EditorUiBindingPayload::MenuAction { action_id } if action_id == HISTORY_REDO_ACTION => {
            Ok(HISTORY_REDO_CLI_BINDING)
        }
        _ => Err(format!(
            "unsupported retained-host automation binding '{}'; supported bindings are SelectionCommand::SelectSceneNode, Inspector transform X commits, workbench.project.save, and workbench.history undo/redo",
            binding.native_binding()
        )),
    }
}

pub(super) fn invoke_supported_binding(
    pane_surface_host: &PaneSurfaceHostContext,
    host: &std::rc::Rc<std::cell::RefCell<RetainedEditorHost>>,
    binding: &EditorUiBinding,
) -> Result<(), String> {
    canonical_cli_binding_path(binding)?;
    match binding.payload() {
        EditorUiBindingPayload::SelectionCommand(SelectionCommand::SelectSceneNode { node_id }) => {
            let (width, height, x, y) = hierarchy_pointer_coordinates(host, *node_id)?;
            pane_surface_host.invoke_hierarchy_pointer_clicked(x, y, width, height);
        }
        EditorUiBindingPayload::InspectorFieldBatch { changes, .. } => {
            let (control_id, binding_id, value) = transform_x_commit(changes)?;
            pane_surface_host.invoke_surface_control_edited(
                control_id.into(),
                binding_id.into(),
                value.into(),
            );
        }
        EditorUiBindingPayload::MenuAction { action_id }
            if action_id == "workbench.project.save" =>
        {
            pane_surface_host.invoke_surface_control_clicked(
                SAVE_PROJECT_CONTROL.into(),
                SAVE_PROJECT_ACTION.into(),
            );
        }
        EditorUiBindingPayload::MenuAction { action_id }
            if action_id == HISTORY_UNDO_ACTION || action_id == HISTORY_REDO_ACTION =>
        {
            pane_surface_host
                .invoke_surface_control_clicked(HISTORY_CONTROL.into(), action_id.as_str().into());
        }
        _ => unreachable!("canonical retained-host automation bindings must have a callback route"),
    }
    Ok(())
}

fn hierarchy_pointer_coordinates(
    host: &std::rc::Rc<std::cell::RefCell<RetainedEditorHost>>,
    node_id: u64,
) -> Result<(f32, f32, f32, f32), String> {
    let mut host = host.borrow_mut();
    host.refresh_ui();
    let width = host.hierarchy_pointer_size.width;
    let height = host.hierarchy_pointer_size.height;
    let rows = host.prepare_hierarchy_pointer_target(width, height, false);
    let index = rows
        .iter()
        .position(|row| row.entity == node_id)
        .ok_or_else(|| {
            format!("requested scene node {node_id} has no authoritative hierarchy row")
        })?;
    let metrics = crate::ui::retained_host::hierarchy_pointer::current_hierarchy_row_metrics();
    let x = metrics.row_x + 1.0;
    let y = crate::ui::retained_host::hierarchy_pointer::hierarchy_row_y(
        metrics,
        index,
        host.hierarchy_pointer_state.scroll_offset,
    ) + metrics.row_height * 0.5;
    if width <= 0.0 || height <= 0.0 || x >= width || y < 0.0 || y >= height {
        return Err(format!(
            "requested scene node {node_id} resolved to hierarchy row {index}, but that retained surface row is outside its current callback bounds {width}x{height}"
        ));
    }
    Ok((width, height, x, y))
}

fn transform_x_commit(
    changes: &[crate::core::editor_event::InspectorFieldChange],
) -> Result<(&'static str, &'static str, String), String> {
    let [change] = changes else {
        return Err(
            "retained-host automation requires exactly one Inspector transform X field change"
                .into(),
        );
    };
    let (control_id, binding_id) = match change.field_id.as_str() {
        POSITION_X_FIELD => (POSITION_X_CONTROL, POSITION_X_BINDING),
        SCALE_X_FIELD => (SCALE_X_CONTROL, SCALE_X_BINDING),
        field_id => {
            return Err(format!(
                "unsupported Inspector field `{field_id}`; retained-host automation supports only transform.translation.x and transform.scale.x"
            ));
        }
    };
    let UiBindingValue::Float(value) = &change.value else {
        return Err(format!(
            "Inspector field `{}` must use a typed finite scalar value",
            change.field_id
        ));
    };
    if !value.is_finite() || *value < f32::MIN as f64 || *value > f32::MAX as f64 {
        return Err(format!(
            "Inspector field `{}` must use a typed finite scalar value",
            change.field_id
        ));
    }
    Ok((control_id, binding_id, (*value as f32).to_string()))
}
