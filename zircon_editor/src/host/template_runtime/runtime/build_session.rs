use crate::host::template_runtime::builtin::{
    builtin_component_descriptors, builtin_template_bindings, builtin_template_documents,
};

use super::runtime_host::{EditorUiHostRuntime, EditorUiHostRuntimeError};

pub(super) fn load_builtin_host_templates(
    runtime: &mut EditorUiHostRuntime,
) -> Result<(), EditorUiHostRuntimeError> {
    if runtime.builtin_host_templates_loaded {
        return Ok(());
    }

    for descriptor in builtin_component_descriptors() {
        runtime.register_component(descriptor)?;
    }

    for (document_id, template) in builtin_template_documents() {
        runtime.register_document_source(document_id, template)?;
    }

    for (binding_id, binding) in builtin_template_bindings() {
        runtime.register_binding(binding_id, binding)?;
    }

    runtime.builtin_host_templates_loaded = true;
    Ok(())
}
