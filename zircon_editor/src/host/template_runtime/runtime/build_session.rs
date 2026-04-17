use zircon_ui::UiTemplateLoader;

use crate::host::template_runtime::builtin::{
    builtin_component_descriptors, builtin_template_bindings, builtin_template_documents,
};

use super::runtime_host::{EditorUiHostRuntime, EditorUiHostRuntimeError};

pub(super) fn load_builtin_workbench_shell(
    runtime: &mut EditorUiHostRuntime,
) -> Result<(), EditorUiHostRuntimeError> {
    if runtime.builtin_workbench_loaded {
        return Ok(());
    }

    for descriptor in builtin_component_descriptors() {
        runtime.register_component(descriptor)?;
    }

    for (document_id, template) in builtin_template_documents() {
        let document = UiTemplateLoader::load_toml_str(template)?;
        runtime
            .template_registry
            .register_document(document_id, document)
            .map_err(EditorUiHostRuntimeError::from)?;
    }

    for (binding_id, binding) in builtin_template_bindings() {
        runtime.register_binding(binding_id, binding)?;
    }

    runtime.builtin_workbench_loaded = true;
    Ok(())
}
