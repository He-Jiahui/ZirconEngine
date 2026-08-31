use std::collections::BTreeMap;

use toml::Value;
use zircon_runtime_interface::ui::style::{UiRgbaColor, UiStyleColor};
use zircon_runtime_interface::ui::v2::{
    UiV2AssetDocument, UiV2ResolvedStyle, UiV2StyleDeclarationBlock,
};

pub(super) fn merge_block_with_token_sources(
    style: &mut UiV2ResolvedStyle,
    block: &UiV2StyleDeclarationBlock,
    document: &UiV2AssetDocument,
) {
    merge_value_map_with_token_sources(
        &mut style.self_values,
        &mut style.style_tokens,
        None,
        &block.self_values,
        document,
    );
    merge_value_map_with_token_sources(
        &mut style.slot,
        &mut style.style_tokens,
        Some("slot"),
        &block.slot,
        document,
    );
}

pub(super) fn style_token_sources_for_block(
    block: &UiV2StyleDeclarationBlock,
    document: &UiV2AssetDocument,
) -> BTreeMap<String, String> {
    let mut tokens = BTreeMap::new();
    collect_value_map_token_sources(None, &block.self_values, &mut tokens, document);
    collect_value_map_token_sources(Some("slot"), &block.slot, &mut tokens, document);
    tokens
}

fn merge_value_map_with_token_sources(
    target: &mut BTreeMap<String, Value>,
    style_tokens: &mut BTreeMap<String, String>,
    prefix: Option<&str>,
    values: &BTreeMap<String, Value>,
    document: &UiV2AssetDocument,
) {
    for (key, value) in values {
        let path = style_token_path(prefix, key);
        remove_style_token_sources(style_tokens, &path);
        collect_value_token_sources(&path, value, style_tokens, document);
        let _ = target.insert(key.clone(), value.clone());
    }
}

fn collect_value_map_token_sources(
    prefix: Option<&str>,
    values: &BTreeMap<String, Value>,
    style_tokens: &mut BTreeMap<String, String>,
    document: &UiV2AssetDocument,
) {
    for (key, value) in values {
        let path = style_token_path(prefix, key);
        collect_value_token_sources(&path, value, style_tokens, document);
    }
}

fn collect_value_token_sources(
    path: &str,
    value: &Value,
    style_tokens: &mut BTreeMap<String, String>,
    document: &UiV2AssetDocument,
) {
    match value {
        Value::String(raw) => {
            if let Some(source) = resolved_token_source(raw, document, 0) {
                let _ = style_tokens.insert(path.to_string(), source);
            }
        }
        Value::Array(values) => {
            for (index, value) in values.iter().enumerate() {
                collect_value_token_sources(
                    &format!("{path}[{index}]"),
                    value,
                    style_tokens,
                    document,
                );
            }
        }
        Value::Table(values) => {
            for (key, value) in values {
                collect_value_token_sources(
                    &format!("{path}.{key}"),
                    value,
                    style_tokens,
                    document,
                );
            }
        }
        _ => {}
    }
}

pub(super) fn style_token_path(prefix: Option<&str>, key: &str) -> String {
    if let Some(prefix) = prefix {
        format!("{prefix}.{key}")
    } else {
        key.to_string()
    }
}

pub(super) fn remove_style_token_sources(style_tokens: &mut BTreeMap<String, String>, path: &str) {
    style_tokens.retain(|key, _| !style_token_path_is_at_or_below(key, path));
}

fn style_token_path_is_at_or_below(candidate: &str, path: &str) -> bool {
    candidate == path
        || candidate
            .strip_prefix(path)
            .is_some_and(|suffix| suffix.starts_with('.') || suffix.starts_with('['))
}

pub(super) fn resolve_value_map(
    values: &mut BTreeMap<String, Value>,
    tokens: &BTreeMap<String, Value>,
    theme: Option<&crate::ui::theme::UiThemeRegistry>,
    depth: usize,
) {
    for value in values.values_mut() {
        resolve_value(value, tokens, theme, depth);
    }
}

fn resolve_value(
    value: &mut Value,
    tokens: &BTreeMap<String, Value>,
    theme: Option<&crate::ui::theme::UiThemeRegistry>,
    depth: usize,
) {
    if depth >= 8 {
        return;
    }
    match value {
        Value::String(raw) => {
            if let Some(theme_value) = theme.and_then(|theme| theme_value(raw, theme)) {
                *value = theme_value;
                return;
            }
            if let Some(replacement) = token_name(raw).and_then(|token| tokens.get(token).cloned())
            {
                *value = replacement;
                resolve_value(value, tokens, theme, depth + 1);
            }
        }
        Value::Array(values) => {
            for value in values {
                resolve_value(value, tokens, theme, depth + 1);
            }
        }
        Value::Table(table) => {
            for (_, value) in table.iter_mut() {
                resolve_value(value, tokens, theme, depth + 1);
            }
        }
        _ => {}
    }
}

fn theme_value(raw: &str, theme: &crate::ui::theme::UiThemeRegistry) -> Option<Value> {
    let role = theme_role(raw)?;
    let color = theme.resolve_role(role)?;
    style_color_value(&color)
}

fn resolved_token_source(raw: &str, document: &UiV2AssetDocument, depth: usize) -> Option<String> {
    if depth >= 8 {
        return None;
    }
    if let Some(theme_source) = theme_role(raw) {
        return Some(theme_source_name(theme_source));
    }
    let token = token_name(raw)?;
    let token_source = format!("token.{token}");
    let nested_source = document.tokens.get(token).and_then(|value| {
        value
            .as_str()
            .and_then(|raw| resolved_token_source(raw, document, depth + 1))
    });
    Some(
        nested_source
            .map(|nested| format!("{token_source} -> {nested}"))
            .unwrap_or(token_source),
    )
}

fn theme_role(raw: &str) -> Option<&str> {
    let unwrapped = raw
        .strip_prefix("var(")
        .and_then(|value| value.strip_suffix(')'))
        .unwrap_or(raw);
    let role = unwrapped.strip_prefix('$').unwrap_or(unwrapped);
    (role.starts_with("theme.") || role.starts_with("theme:") || role.starts_with("palette."))
        .then_some(role)
}

fn theme_source_name(role: &str) -> String {
    let normalized = role
        .strip_prefix('$')
        .unwrap_or(role)
        .strip_prefix("theme:")
        .map(|role| format!("theme.{role}"))
        .unwrap_or_else(|| {
            if role.starts_with("theme.") {
                role.to_string()
            } else {
                format!("theme.{role}")
            }
        });
    normalized
}

fn style_color_value(color: &UiStyleColor) -> Option<Value> {
    match color {
        UiStyleColor::Rgba(color) => Some(Value::String(rgba_hex(*color))),
        UiStyleColor::Transparent => Some(Value::String("transparent".to_string())),
        UiStyleColor::Inherit => Some(Value::String("inherit".to_string())),
        UiStyleColor::Role(_) => None,
    }
}

fn rgba_hex(color: UiRgbaColor) -> String {
    let [red, green, blue, alpha] = color.to_u8();
    if alpha == 255 {
        format!("#{red:02x}{green:02x}{blue:02x}")
    } else {
        format!("#{red:02x}{green:02x}{blue:02x}{alpha:02x}")
    }
}

fn token_name(value: &str) -> Option<&str> {
    value
        .strip_prefix('$')
        .filter(|token| !token.is_empty())
        .or_else(|| {
            value
                .strip_prefix("var(")
                .and_then(|value| value.strip_suffix(')'))
        })
}

#[cfg(test)]
#[path = "tokens/allocation_free_path_match_tests.rs"]
mod allocation_free_path_match_tests;
