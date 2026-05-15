use std::collections::BTreeMap;

use zircon_runtime_interface::ui::binding::UiEventKind;

use crate::ui::binding::EditorUiBinding;
use crate::ui::control::EditorUiControlService;
use crate::ui::template_runtime::{
    EditorUiHostRuntime, EditorUiHostRuntimeError, RetainedUiHostProjection, RetainedUiProjection,
};

pub(crate) fn build_bindings_by_id(
    projection: &RetainedUiProjection,
) -> BTreeMap<String, EditorUiBinding> {
    projection
        .bindings
        .iter()
        .map(|binding| (binding.binding_id.clone(), binding.binding.clone()))
        .collect::<BTreeMap<_, _>>()
}

pub(crate) fn binding_for_control<'a>(
    bindings_by_id: &'a BTreeMap<String, EditorUiBinding>,
    host_projection: &'a RetainedUiHostProjection,
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

#[cfg(test)]
pub(crate) fn project_builtin_surface(
    document_id: &str,
) -> Result<(BTreeMap<String, EditorUiBinding>, RetainedUiHostProjection), EditorUiHostRuntimeError>
{
    let runtime = load_builtin_runtime()?;
    project_builtin_surface_with_runtime(&runtime, document_id)
}

pub(crate) fn project_builtin_surface_with_runtime(
    runtime: &EditorUiHostRuntime,
    document_id: &str,
) -> Result<(BTreeMap<String, EditorUiBinding>, RetainedUiHostProjection), EditorUiHostRuntimeError>
{
    let projection = project_builtin_document_with_runtime(runtime, document_id)?;
    let bindings_by_id = build_bindings_by_id(&projection);
    let host_projection = runtime.build_retained_host_projection(&projection)?;
    Ok((bindings_by_id, host_projection))
}

#[cfg(test)]
pub(crate) fn load_builtin_runtime() -> Result<EditorUiHostRuntime, EditorUiHostRuntimeError> {
    let mut runtime = EditorUiHostRuntime::default();
    runtime.load_builtin_host_templates()?;
    Ok(runtime)
}

pub(crate) fn load_builtin_runtime_for_documents(
    document_ids: &[&str],
) -> Result<EditorUiHostRuntime, EditorUiHostRuntimeError> {
    let mut runtime = EditorUiHostRuntime::default();
    runtime.load_builtin_host_templates_for_document_ids(document_ids)?;
    Ok(runtime)
}

pub(crate) fn project_builtin_document_with_runtime(
    runtime: &EditorUiHostRuntime,
    document_id: &str,
) -> Result<RetainedUiProjection, EditorUiHostRuntimeError> {
    let mut projection = runtime.project_document(document_id)?;
    let mut route_service = EditorUiControlService::default();
    runtime.register_projection_routes(&mut route_service, &mut projection)?;
    Ok(projection)
}
