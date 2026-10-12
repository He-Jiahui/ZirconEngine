use crate::ui::template_runtime::RetainedUiHostBindingProjection;

use super::binding_ids::{showcase_action_id_for_binding_id, showcase_binding_with_suffix};

pub(in super::super) fn preferred_showcase_drag_action_id(
    control_id: &str,
    bindings: &[RetainedUiHostBindingProjection],
) -> Option<String> {
    let suffix = drag_action_suffix(control_id)?;
    showcase_binding_with_suffix(bindings, suffix)
        .map(|binding| showcase_action_id_for_binding_id(&binding.binding_id))
}

pub(in super::super) fn preferred_showcase_pointer_drag_action_id(
    control_id: &str,
    event_suffix: &str,
    bindings: &[RetainedUiHostBindingProjection],
) -> Option<String> {
    let suffix = pointer_drag_action_suffix(control_id, event_suffix)?;
    showcase_binding_with_suffix(bindings, suffix)
        .map(|binding| showcase_action_id_for_binding_id(&binding.binding_id))
}

fn drag_action_suffix(control_id: &str) -> Option<&'static str> {
    match control_id {
        "NumberFieldDemo" => Some("NumberFieldDragUpdate"),
        "RangeFieldDemo" => Some("RangeFieldDragUpdate"),
        "SliderDemo" => Some("SliderDragUpdate"),
        "RangeSliderDemo" => Some("RangeSliderDragUpdate"),
        _ => None,
    }
}

fn pointer_drag_action_suffix(control_id: &str, event_suffix: &str) -> Option<&'static str> {
    match (control_id, event_suffix) {
        ("NumberFieldDemo", "DragBegin") => Some("NumberFieldDragBegin"),
        ("NumberFieldDemo", "DragEnd") => Some("NumberFieldDragEnd"),
        _ => None,
    }
}
