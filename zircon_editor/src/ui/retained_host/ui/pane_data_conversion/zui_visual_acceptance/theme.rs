use toml::Value;
use zircon_runtime_interface::ui::design_tokens::EditorDesignTokens;

pub(super) fn verify(source: &Value, tokens: &EditorDesignTokens) -> Result<(), String> {
    let names = source
        .get("names")
        .and_then(Value::as_table)
        .ok_or("host theme has no canonical token names")?;
    let actual = tokens.cascade_token_values();
    for group in ["palette", "typography", "controls", "chrome", "density"] {
        let mappings = names
            .get(group)
            .and_then(Value::as_table)
            .ok_or_else(|| format!("host theme has no {group} names"))?;
        let values = source
            .get(group)
            .and_then(Value::as_table)
            .ok_or_else(|| format!("host theme has no {group} values"))?;
        for (field, name) in mappings {
            let name = name
                .as_str()
                .ok_or("host theme token name must be a string")?;
            let expected = if group == "palette"
                && field.starts_with("surface_")
                && field[8..].parse::<usize>().is_ok()
            {
                values
                    .get("surface")
                    .and_then(Value::as_array)
                    .and_then(|items| items.get(field[8..].parse::<usize>().ok()?))
            } else {
                values.get(field)
            }
            .ok_or_else(|| format!("host theme missing {group}.{field}"))?;
            let current = actual
                .get(name)
                .ok_or_else(|| format!("Settings authority has no {name}"))?;
            if !matches_value(expected, current) {
                return Err(format!(
                    "host theme differs at {name}: requested {expected}, Settings authority {current}"
                ));
            }
        }
    }
    let states = source
        .get("state_roles")
        .and_then(Value::as_table)
        .ok_or("missing theme state roles")?;
    for (state, role) in states {
        let name = names
            .get("state_roles")
            .and_then(|names| names.get(state))
            .and_then(Value::as_str)
            .ok_or_else(|| format!("missing theme state name: {state}"))?;
        let palette = role
            .as_str()
            .and_then(|role| names.get("palette")?.get(role))
            .and_then(Value::as_str)
            .ok_or_else(|| format!("unknown theme state role: {state} = {role}"))?;
        if actual.get(name) != Some(&Value::String(format!("${palette}"))) {
            return Err(format!(
                "host theme state {name} differs from Settings authority"
            ));
        }
    }
    Ok(())
}

fn matches_value(expected: &Value, current: &Value) -> bool {
    match (expected, current) {
        (Value::Float(a), Value::Float(b)) => (*a - *b).abs() < 0.00001,
        (Value::Integer(a), Value::Float(b)) => (*a as f64 - *b).abs() < 0.00001,
        (Value::Float(a), Value::Integer(b)) => (*a - *b as f64).abs() < 0.00001,
        (Value::String(a), Value::String(b))
            if matches!(a.len(), 7 | 9)
                && a.len() == b.len()
                && a.starts_with('#')
                && b.starts_with('#')
                && a.as_bytes()[1..].iter().all(u8::is_ascii_hexdigit)
                && b.as_bytes()[1..].iter().all(u8::is_ascii_hexdigit) =>
        {
            a.eq_ignore_ascii_case(b)
        }
        _ => expected == current,
    }
}

#[cfg(test)]
include!("tests/theme_cases.rs");
