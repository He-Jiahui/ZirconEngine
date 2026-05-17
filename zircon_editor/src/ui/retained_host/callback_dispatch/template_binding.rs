use crate::core::editor_event::EditorEventRuntime;
use crate::ui::binding::{EditorUiBinding, EditorUiBindingPayload};
use crate::ui::retained_host::{event_bridge::UiHostEventEffects, HostInvalidationMask};
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
    if let Some(effects) = material_lab_feedback_effects(&binding) {
        return Ok(effects);
    }
    dispatch_editor_binding(runtime, binding)
}

fn material_lab_feedback_effects(binding: &EditorUiBinding) -> Option<UiHostEventEffects> {
    let EditorUiBindingPayload::Custom(call) = binding.payload() else {
        return None;
    };
    if call.symbol != "MaterialComponentLab" {
        return None;
    }

    let mut effects = UiHostEventEffects::default();
    effects.merge_dirty_domains(HostInvalidationMask::PAINT_ONLY);
    Some(effects)
}

#[cfg(test)]
mod tests {
    use super::*;
    use zircon_runtime_interface::ui::binding::UiBindingCall;

    #[test]
    fn material_component_lab_custom_binding_is_paint_only_feedback() {
        let binding = EditorUiBinding::new(
            "MaterialComponentLab",
            "MaterialLabButtons",
            crate::ui::binding::EditorUiEventKind::Click,
            EditorUiBindingPayload::Custom(UiBindingCall::new("MaterialComponentLab")),
        );

        let effects =
            material_lab_feedback_effects(&binding).expect("Material Lab payload should match");
        let dirty_domains = effects.dirty_domains();

        assert!(dirty_domains.contains(HostInvalidationMask::PAINT_ONLY));
        assert!(!dirty_domains.requires_presentation());
        assert!(!dirty_domains.requires_layout());
    }
}
