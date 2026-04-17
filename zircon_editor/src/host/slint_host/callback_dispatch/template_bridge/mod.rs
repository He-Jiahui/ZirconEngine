mod asset_surface;
mod floating_window_source;
mod inspector_surface;
mod pane_surface;
mod viewport_toolbar;
mod welcome_surface;
mod workbench;

use std::collections::BTreeMap;

use zircon_editor_ui::{EditorUiBinding, EditorUiControlService};
use zircon_ui::UiEventKind;

use crate::host::template_runtime::{
    EditorUiHostRuntime, EditorUiHostRuntimeError, SlintUiHostProjection, SlintUiProjection,
};

pub(crate) use asset_surface::BuiltinAssetSurfaceTemplateBridge;
pub(crate) use floating_window_source::{
    BuiltinFloatingWindowSourceFrames, BuiltinFloatingWindowSourceTemplateBridge,
};
pub(crate) use inspector_surface::BuiltinInspectorSurfaceTemplateBridge;
pub(crate) use pane_surface::BuiltinPaneSurfaceTemplateBridge;
pub(crate) use viewport_toolbar::BuiltinViewportToolbarTemplateBridge;
pub(crate) use welcome_surface::BuiltinWelcomeSurfaceTemplateBridge;
pub(crate) use workbench::{BuiltinWorkbenchRootShellFrames, BuiltinWorkbenchTemplateBridge};

fn build_bindings_by_id(projection: &SlintUiProjection) -> BTreeMap<String, EditorUiBinding> {
    projection
        .bindings
        .iter()
        .map(|binding| (binding.binding_id.clone(), binding.binding.clone()))
        .collect::<BTreeMap<_, _>>()
}

fn binding_for_control<'a>(
    bindings_by_id: &'a BTreeMap<String, EditorUiBinding>,
    host_projection: &'a SlintUiHostProjection,
    control_id: &str,
    event_kind: UiEventKind,
) -> Option<&'a EditorUiBinding> {
    let binding_id = host_projection
        .node_by_control_id(control_id)?
        .routes
        .iter()
        .find(|route| route.event_kind == event_kind)?
        .binding_id
        .as_str();
    bindings_by_id.get(binding_id)
}

fn project_builtin_surface(
    document_id: &str,
) -> Result<(BTreeMap<String, EditorUiBinding>, SlintUiHostProjection), EditorUiHostRuntimeError> {
    let mut runtime = EditorUiHostRuntime::default();
    runtime.load_builtin_workbench_shell()?;

    let mut projection = runtime.project_document(document_id)?;
    let mut route_service = EditorUiControlService::default();
    runtime.register_projection_routes(&mut route_service, &mut projection)?;

    let bindings_by_id = build_bindings_by_id(&projection);
    let host_projection = runtime.build_slint_host_projection(&projection)?;
    Ok((bindings_by_id, host_projection))
}

fn load_builtin_runtime_projection(
    document_id: &str,
) -> Result<(EditorUiHostRuntime, SlintUiProjection), EditorUiHostRuntimeError> {
    let mut runtime = EditorUiHostRuntime::default();
    runtime.load_builtin_workbench_shell()?;

    let mut projection = runtime.project_document(document_id)?;
    let mut route_service = EditorUiControlService::default();
    runtime.register_projection_routes(&mut route_service, &mut projection)?;
    Ok((runtime, projection))
}
