use zircon_runtime_interface::ui::component::{UiComponentAdapterResult, UiValue};

use super::super::support::showcase_binding;
use crate::ui::template_runtime::{EditorUiHostRuntime, UiComponentShowcaseDemoEventInput};

pub(super) fn apply_showcase_binding(
    runtime: &mut EditorUiHostRuntime,
    binding_id: &str,
    input: UiComponentShowcaseDemoEventInput,
) -> UiComponentAdapterResult {
    let binding = showcase_binding(runtime, binding_id);
    runtime
        .apply_showcase_demo_binding(&binding, input)
        .unwrap()
}
