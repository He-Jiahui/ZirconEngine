use crate::ui::binding::EditorUiBinding;
use crate::ui::template_runtime::showcase_demo_state::{
    resolve_showcase_component_event, UiComponentShowcaseDemoError,
    UiComponentShowcaseDemoEventInput, UiComponentShowcaseDemoState,
};
use zircon_runtime_interface::ui::component::{
    UiComponentAdapterResult, UiComponentProjectionPatch, UiValue,
};

pub(crate) fn apply_showcase_component_binding(
    state: &mut UiComponentShowcaseDemoState,
    binding: &EditorUiBinding,
    input: UiComponentShowcaseDemoEventInput,
) -> Result<UiComponentAdapterResult, UiComponentShowcaseDemoError> {
    let resolved = resolve_showcase_component_event(binding, input)?;
    let changed_value = state.apply_component_event_envelope(
        &resolved.action,
        &resolved.envelope,
        resolved.changed_property.as_deref(),
    )?;
    let control_id = resolved.envelope.control_id;
    let changed_property = resolved.changed_property;

    let mut patch = UiComponentProjectionPatch::new(control_id);
    if let (Some(property), Some(value)) = (changed_property, changed_value) {
        let value_text = value.display_text();
        patch = patch
            .with_state_value(property, value)
            .with_attribute("value_text", UiValue::String(value_text));
    }

    Ok(UiComponentAdapterResult::changed().with_patch(patch))
}

#[cfg(test)]
#[path = "tests/showcase_performance_tests.rs"]
mod performance_tests;
