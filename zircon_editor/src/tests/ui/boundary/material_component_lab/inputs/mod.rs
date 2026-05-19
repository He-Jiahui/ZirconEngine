use zircon_runtime_interface::ui::v2::UiV2NodeDefinition;

mod checkbox;
mod numeric;
mod radio;
mod selection;
mod switch;
mod text;
mod toggle;

fn assert_non_dispatchable_child(node: &UiV2NodeDefinition, node_id: &str, component: &str) {
    for prop in [
        "input_interactive",
        "input_clickable",
        "input_hoverable",
        "input_focusable",
    ] {
        assert_eq!(
            node.props.get(prop).and_then(|value| value.as_bool()),
            Some(false),
            "{component} child `{node_id}` should leave dispatchability on the visible sample"
        );
    }
}

fn string_array_prop(value: Option<&toml::Value>) -> Vec<&str> {
    value
        .and_then(|value| value.as_array())
        .map(|values| values.iter().filter_map(|value| value.as_str()).collect())
        .unwrap_or_default()
}
