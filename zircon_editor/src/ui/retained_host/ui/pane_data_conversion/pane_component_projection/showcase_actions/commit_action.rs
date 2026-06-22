use crate::ui::template_runtime::RetainedUiHostBindingProjection;

use super::binding_ids::{showcase_action_id_for_binding_id, showcase_binding_with_suffix};

pub(in super::super) fn preferred_showcase_commit_action_id(
    control_id: &str,
    bindings: &[RetainedUiHostBindingProjection],
) -> Option<String> {
    let suffix = commit_action_suffix(control_id)?;
    showcase_binding_with_suffix(bindings, suffix)
        .map(|binding| showcase_action_id_for_binding_id(&binding.binding_id))
}

fn commit_action_suffix(control_id: &str) -> Option<&'static str> {
    match control_id {
        "InputFieldDemo" => Some("InputFieldCommitted"),
        "TextFieldDemo" => Some("TextFieldCommitted"),
        "NumberFieldDemo" => Some("NumberFieldCommitted"),
        "RangeFieldDemo" => Some("RangeFieldCommitted"),
        _ => None,
    }
}
