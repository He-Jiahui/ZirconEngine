use crate::ui::template_runtime::RetainedUiHostBindingProjection;

const SHOWCASE_BINDING_PREFIX: &str = "UiComponentShowcase/";

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
    format!("ui_component_showcase.{}", camel_to_snake(suffix))
}

fn binding_path_action_id(binding_id: &str) -> String {
    binding_id
        .split(['/', '.', ':'])
        .filter(|segment| !segment.is_empty())
        .map(camel_to_snake)
        .collect::<Vec<_>>()
        .join(".")
}

fn camel_to_snake(value: &str) -> String {
    let mut output = String::new();
    let mut previous_was_separator = true;
    for ch in value.chars() {
        if ch.is_ascii_alphanumeric() {
            if ch.is_ascii_uppercase() && !previous_was_separator && !output.ends_with('_') {
                output.push('_');
            }
            output.push(ch.to_ascii_lowercase());
            previous_was_separator = false;
        } else if !output.ends_with('_') {
            output.push('_');
            previous_was_separator = true;
        }
    }
    output.trim_matches('_').to_string()
}
