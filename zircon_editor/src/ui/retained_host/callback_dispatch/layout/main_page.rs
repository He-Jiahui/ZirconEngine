use crate::ui::host::EditorHostEventController;
use crate::ui::retained_host::event_bridge::UiHostEventEffects;

use super::super::{common::dispatch_editor_binding, BuiltinHostWindowTemplateBridge};

pub(crate) fn dispatch_builtin_host_page_activation(
    runtime: &EditorHostEventController,
    bridge: &BuiltinHostWindowTemplateBridge,
    page_id: &str,
) -> Option<Result<UiHostEventEffects, String>> {
    let binding = bridge.host_page_activation_binding(page_id)?;
    Some(dispatch_editor_binding(runtime, binding))
}
