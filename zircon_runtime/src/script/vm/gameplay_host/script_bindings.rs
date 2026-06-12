pub(super) const SCRIPT_BINDINGS_COMPONENT: &str = "script.bindings";

pub(super) fn script_binding_property_matches(
    bindings: &serde_json::Value,
    property: &str,
    expected_value: &str,
) -> bool {
    bindings.as_array().map_or(false, |bindings| {
        bindings.iter().any(|binding| {
            binding
                .get("enabled")
                .and_then(serde_json::Value::as_bool)
                .unwrap_or(true)
                && binding
                    .get("properties")
                    .and_then(|properties| properties.get(property))
                    .map(|value| json_value_matches(value, expected_value))
                    .unwrap_or(false)
        })
    })
}

pub(super) fn apply_damage_to_script_health(
    bindings: &mut serde_json::Value,
    damage: f64,
) -> Option<f64> {
    update_script_health(bindings, |current| (current - damage).max(0.0))
}

pub(super) fn apply_heal_to_script_health(
    bindings: &mut serde_json::Value,
    amount: f64,
    max_health: f64,
) -> Option<f64> {
    update_script_health(bindings, |current| (current + amount).min(max_health))
}

pub(super) fn script_binding_number(bindings: &serde_json::Value, property: &str) -> Option<f64> {
    bindings.as_array()?.iter().find_map(|binding| {
        if !binding
            .get("enabled")
            .and_then(serde_json::Value::as_bool)
            .unwrap_or(true)
        {
            return None;
        }
        binding
            .get("properties")
            .and_then(|properties| properties.get(property))
            .and_then(serde_json::Value::as_f64)
    })
}

fn update_script_health(
    bindings: &mut serde_json::Value,
    update: impl FnOnce(f64) -> f64,
) -> Option<f64> {
    for binding in bindings.as_array_mut()? {
        if !binding
            .get("enabled")
            .and_then(serde_json::Value::as_bool)
            .unwrap_or(true)
        {
            continue;
        }
        let Some(properties) = binding
            .get_mut("properties")
            .and_then(serde_json::Value::as_object_mut)
        else {
            continue;
        };
        let Some(health) = properties.get_mut("hp") else {
            continue;
        };
        let updated = update(health.as_f64()?);
        let updated_number = serde_json::Number::from_f64(updated)?;
        *health = serde_json::Value::Number(updated_number);
        return Some(updated);
    }
    None
}

fn json_value_matches(value: &serde_json::Value, expected_value: &str) -> bool {
    match value {
        serde_json::Value::String(value) => value == expected_value,
        serde_json::Value::Bool(value) => {
            (*value && expected_value == "true") || (!*value && expected_value == "false")
        }
        serde_json::Value::Number(value) => value.to_string() == expected_value,
        _ => false,
    }
}
