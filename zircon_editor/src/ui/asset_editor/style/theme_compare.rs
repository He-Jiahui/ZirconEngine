use std::collections::BTreeMap;

use toml::Value;
use zircon_runtime::ui::template::{UiAssetDocument, UiStyleDeclarationBlock};

use super::theme_authoring::can_promote_local_theme_to_external_style_asset;

pub(crate) fn build_theme_compare_items(
    document: &UiAssetDocument,
    imported_styles: &BTreeMap<String, UiAssetDocument>,
    selected_key: Option<&str>,
) -> Vec<String> {
    let selected_key = selected_key.unwrap_or_else(|| {
        if can_promote_local_theme_to_external_style_asset(document) {
            "local"
        } else {
            ""
        }
    });
    if selected_key.is_empty() {
        return Vec::new();
    }
    if selected_key == "local" {
        return compare_local_against_imports(document, imported_styles);
    }
    let Some(imported) = imported_styles.get(selected_key) else {
        return Vec::new();
    };
    compare_imported_against_local(imported, document)
}

fn compare_imported_against_local(
    imported: &UiAssetDocument,
    local: &UiAssetDocument,
) -> Vec<String> {
    let mut items = Vec::new();
    for token_name in imported.tokens.keys() {
        match (
            imported.tokens.get(token_name),
            local.tokens.get(token_name),
        ) {
            (Some(imported_value), Some(local_value)) if imported_value == local_value => items
                .push(format!(
                    "shared • token • {token_name} = {}",
                    format_value(imported_value)
                )),
            (Some(imported_value), Some(local_value)) => items.push(format!(
                "shadowed by local • token • {token_name} • imported = {} • local = {}",
                format_value(imported_value),
                format_value(local_value)
            )),
            (Some(imported_value), None) => items.push(format!(
                "imported-only • token • {token_name} = {}",
                format_value(imported_value)
            )),
            _ => {}
        }
    }
    for token_name in local.tokens.keys() {
        if imported.tokens.contains_key(token_name) {
            continue;
        }
        if let Some(local_value) = local.tokens.get(token_name) {
            items.push(format!(
                "local-only • token • {token_name} = {}",
                format_value(local_value)
            ));
        }
    }

    let imported_rules = selector_rule_blocks(imported);
    let local_rules = selector_rule_blocks(local);
    for (selector, (imported_label, imported_block)) in &imported_rules {
        match local_rules.get(selector) {
            Some((_, local_block)) if local_block == imported_block => {
                items.push(format!("shared • rule • {imported_label}"));
            }
            Some((_, local_block)) => items.push(format!(
                "shadowed by local • rule • {imported_label} • imported {} • local {}",
                format_rule_block(imported_block),
                format_rule_block(local_block)
            )),
            None => items.push(format!(
                "imported-only • rule • {imported_label} • {}",
                format_rule_block(imported_block)
            )),
        }
    }
    for (selector, (local_label, local_block)) in &local_rules {
        if !imported_rules.contains_key(selector) {
            items.push(format!(
                "local-only • rule • {local_label} • {}",
                format_rule_block(local_block)
            ));
        }
    }
    items
}

fn compare_local_against_imports(
    local: &UiAssetDocument,
    imported_styles: &BTreeMap<String, UiAssetDocument>,
) -> Vec<String> {
    let mut items = Vec::new();
    let mut imported_tokens = BTreeMap::<String, Value>::new();
    for reference in &local.imports.styles {
        let Some(imported) = imported_styles.get(reference) else {
            continue;
        };
        for (name, value) in &imported.tokens {
            imported_tokens.insert(name.clone(), value.clone());
        }
    }

    for (token_name, local_value) in &local.tokens {
        match imported_tokens.get(token_name) {
            Some(imported_value) if imported_value == local_value => items.push(format!(
                "shared • token • {token_name} = {}",
                format_value(local_value)
            )),
            Some(imported_value) => items.push(format!(
                "overrides imported • token • {token_name} • imported = {} • local = {}",
                format_value(imported_value),
                format_value(local_value)
            )),
            None => items.push(format!(
                "local-only • token • {token_name} = {}",
                format_value(local_value)
            )),
        }
    }

    for (token_name, imported_value) in imported_tokens {
        if local.tokens.contains_key(&token_name) {
            continue;
        }
        items.push(format!(
            "inherited • token • {token_name} = {}",
            format_value(&imported_value)
        ));
    }

    let local_rules = selector_rule_blocks(local);
    let imported_rule_blocks = aggregated_rule_blocks(local, imported_styles);
    for (selector, (local_label, local_block)) in &local_rules {
        match imported_rule_blocks.get(selector) {
            Some((_, imported_block)) if imported_block == local_block => {
                items.push(format!("shared • rule • {local_label}"));
            }
            Some((_, imported_block)) => items.push(format!(
                "overrides imported • rule • {local_label} • imported {} • local {}",
                format_rule_block(imported_block),
                format_rule_block(local_block)
            )),
            None => items.push(format!(
                "local-only • rule • {local_label} • {}",
                format_rule_block(local_block)
            )),
        }
    }
    for (selector, (imported_label, imported_block)) in imported_rule_blocks {
        if !local_rules.contains_key(&selector) {
            items.push(format!(
                "inherited • rule • {imported_label} • {}",
                format_rule_block(&imported_block)
            ));
        }
    }
    items
}

fn selector_rule_blocks(
    document: &UiAssetDocument,
) -> BTreeMap<String, (String, UiStyleDeclarationBlock)> {
    let mut rules = BTreeMap::new();
    for stylesheet in &document.stylesheets {
        let stylesheet_label = if stylesheet.id.is_empty() {
            "<inline>"
        } else {
            stylesheet.id.as_str()
        };
        for rule in &stylesheet.rules {
            rules.insert(
                rule.selector.clone(),
                (
                    format!("{stylesheet_label} • {}", rule.selector),
                    rule.set.clone(),
                ),
            );
        }
    }
    rules
}

fn aggregated_rule_blocks(
    local: &UiAssetDocument,
    imported_styles: &BTreeMap<String, UiAssetDocument>,
) -> BTreeMap<String, (String, UiStyleDeclarationBlock)> {
    let mut rules = BTreeMap::new();
    for reference in &local.imports.styles {
        let Some(imported) = imported_styles.get(reference) else {
            continue;
        };
        for (selector, block) in selector_rule_blocks(imported) {
            rules.insert(selector, block);
        }
    }
    rules
}

fn format_rule_block(block: &UiStyleDeclarationBlock) -> String {
    let mut entries = Vec::new();
    for (key, value) in &block.self_values {
        push_rule_block_value(&mut entries, format!("self.{key}"), value);
    }
    for (key, value) in &block.slot {
        push_rule_block_value(&mut entries, format!("slot.{key}"), value);
    }
    entries.sort();
    if entries.is_empty() {
        "<empty>".to_string()
    } else {
        entries.join("; ")
    }
}

fn push_rule_block_value(entries: &mut Vec<String>, path: String, value: &Value) {
    match value {
        Value::Table(table) => {
            for (key, nested) in table {
                push_rule_block_value(entries, format!("{path}.{key}"), nested);
            }
        }
        _ => entries.push(format!("{path} = {}", value)),
    }
}

fn format_value(value: &Value) -> String {
    value.to_string()
}
