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
    let control_id = resolved.envelope.control_id.clone();
    let changed_property = resolved.changed_property.clone();
    let changed_value = state.apply_component_event_envelope(
        &resolved.action,
        &resolved.envelope,
        changed_property.as_deref(),
    )?;

    let mut patch = UiComponentProjectionPatch::new(control_id);
    if let (Some(property), Some(value)) = (changed_property, changed_value) {
        patch = patch
            .with_state_value(property, value.clone())
            .with_attribute("value_text", UiValue::String(value.display_text()));
    }

    Ok(UiComponentAdapterResult::changed().with_patch(patch))
}
