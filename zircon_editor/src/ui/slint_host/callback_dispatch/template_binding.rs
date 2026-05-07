use crate::core::editor_event::EditorEventRuntime;
use crate::ui::binding::EditorUiBinding;
use crate::ui::slint_host::event_bridge::UiHostEventEffects;
use crate::ui::template_runtime::builtin::builtin_template_bindings;
use zircon_runtime_interface::ui::binding::UiBindingValue;

use super::common::dispatch_editor_binding;

pub(crate) fn dispatch_builtin_template_binding(
    runtime: &EditorEventRuntime,
    binding_id: &str,
) -> Option<Result<UiHostEventEffects, String>> {
    let binding = builtin_template_bindings().remove(binding_id)?;
    Some(dispatch_editor_binding(runtime, binding))
}

pub(crate) fn dispatch_builtin_template_binding_with_arguments(
    runtime: &EditorEventRuntime,
    binding_id: &str,
    arguments: Vec<UiBindingValue>,
) -> Option<Result<UiHostEventEffects, String>> {
    let binding = builtin_template_bindings().remove(binding_id)?;
    Some(dispatch_template_binding_with_arguments(
        runtime, binding, arguments,
    ))
}

pub(crate) fn dispatch_template_binding_with_arguments(
    runtime: &EditorEventRuntime,
    binding: EditorUiBinding,
    arguments: Vec<UiBindingValue>,
) -> Result<UiHostEventEffects, String> {
    let binding = if arguments.is_empty() {
        binding
    } else {
        binding
            .with_arguments(arguments)
            .map_err(|error| error.to_string())?
    };
    dispatch_editor_binding(runtime, binding)
}
