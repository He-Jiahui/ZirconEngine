use crate::core::editor_event::EditorEventRuntime;
use crate::ui::slint_host::event_bridge::SlintDispatchEffects;
use crate::ui::template_runtime::builtin::builtin_template_bindings;

use super::common::dispatch_editor_binding;

pub(crate) fn dispatch_builtin_template_binding(
    runtime: &EditorEventRuntime,
    binding_id: &str,
) -> Option<Result<SlintDispatchEffects, String>> {
    let binding = builtin_template_bindings().remove(binding_id)?;
    Some(dispatch_editor_binding(runtime, binding))
}
