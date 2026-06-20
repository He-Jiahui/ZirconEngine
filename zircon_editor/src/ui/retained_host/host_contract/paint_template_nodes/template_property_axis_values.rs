#[derive(Debug, PartialEq, Eq)]
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) struct PropertyAxisValue {
    pub(in crate::ui::retained_host::host_contract::paint_template_nodes) axis: String,
    pub(in crate::ui::retained_host::host_contract::paint_template_nodes) value: String,
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn property_axis_values(
    value: &str,
) -> Vec<PropertyAxisValue> {
    let mut values = Vec::new();
    let mut current_axis: Option<String> = None;
    let mut current_value = Vec::new();

    for token in value.split_whitespace() {
        if is_axis_token(token) {
            push_current_axis_value(&mut values, &mut current_axis, &mut current_value);
            current_axis = Some(token.to_string());
        } else if current_axis.is_some() {
            current_value.push(token.to_string());
        }
    }
    push_current_axis_value(&mut values, &mut current_axis, &mut current_value);
    values
}

fn push_current_axis_value(
    values: &mut Vec<PropertyAxisValue>,
    current_axis: &mut Option<String>,
    current_value: &mut Vec<String>,
) {
    let Some(axis) = current_axis.take() else {
        return;
    };
    let value = current_value.join(" ");
    current_value.clear();
    if !value.is_empty() {
        values.push(PropertyAxisValue { axis, value });
    }
}

fn is_axis_token(token: &str) -> bool {
    matches!(token, "X" | "Y" | "Z" | "W")
}
