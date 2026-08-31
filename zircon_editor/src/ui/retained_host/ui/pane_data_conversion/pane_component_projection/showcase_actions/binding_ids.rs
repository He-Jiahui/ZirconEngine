use crate::ui::template_runtime::RetainedUiHostBindingProjection;

use super::super::binding_actions::{binding_path_action_id, push_binding_path_action_id};

const SHOWCASE_BINDING_PREFIX: &str = "UiComponentShowcase/";
const SHOWCASE_ACTION_PREFIX: &str = "ui_component_showcase.";

pub(in super::super) fn showcase_action_id_for_suffix(
    bindings: &[RetainedUiHostBindingProjection],
    suffix: &str,
) -> String {
    showcase_binding_with_suffix(bindings, suffix)
        .map(|binding| showcase_action_id_for_binding_id(&binding.binding_id))
        .unwrap_or_default()
}

pub(super) fn showcase_binding_with_suffix<'a>(
    bindings: &'a [RetainedUiHostBindingProjection],
    suffix: &str,
) -> Option<&'a RetainedUiHostBindingProjection> {
    bindings.iter().find(|binding| {
        binding.binding_id.starts_with(SHOWCASE_BINDING_PREFIX)
            && binding.binding_id.ends_with(suffix)
    })
}

pub(super) fn first_showcase_binding(
    bindings: &[RetainedUiHostBindingProjection],
) -> Option<&RetainedUiHostBindingProjection> {
    bindings
        .iter()
        .find(|binding| binding.binding_id.starts_with(SHOWCASE_BINDING_PREFIX))
}

pub(super) fn showcase_action_id_for_binding_id(binding_id: &str) -> String {
    let Some(suffix) = binding_id.strip_prefix(SHOWCASE_BINDING_PREFIX) else {
        return binding_path_action_id(binding_id);
    };
    let mut action_id = String::with_capacity(SHOWCASE_ACTION_PREFIX.len() + suffix.len());
    action_id.push_str(SHOWCASE_ACTION_PREFIX);
    push_binding_path_action_id(&mut action_id, suffix);
    action_id
}
