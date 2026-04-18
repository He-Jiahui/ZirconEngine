use std::collections::{BTreeMap, BTreeSet};

use toml::Value;
use zircon_ui::{
    UiAssetDocument, UiAssetHeader, UiAssetImports, UiAssetKind, UiStyleDeclarationBlock,
    UiStyleSheet,
};

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct UiAssetExternalStyleDraft {
    pub(crate) asset_id: String,
    pub(crate) document_id: String,
    pub(crate) display_name: String,
}

pub(crate) fn can_promote_local_theme_to_external_style_asset(document: &UiAssetDocument) -> bool {
    !document.tokens.is_empty() || !document.stylesheets.is_empty()
}

pub(crate) fn default_external_style_draft(
    source_asset_id: &str,
    source_display_name: &str,
) -> UiAssetExternalStyleDraft {
    let base_name = theme_base_name(source_asset_id);
    UiAssetExternalStyleDraft {
        asset_id: format!("res://ui/themes/{base_name}_theme.ui.toml"),
        document_id: format!("ui.theme.{base_name}_theme"),
        display_name: theme_display_name(source_display_name, &base_name),
    }
}

pub(crate) fn promote_local_theme_to_external_style_asset(
    document: &mut UiAssetDocument,
    style_asset_id: &str,
    style_document_id: &str,
    display_name: &str,
) -> Option<UiAssetDocument> {
    if !can_promote_local_theme_to_external_style_asset(document) {
        return None;
    }

    let promoted_theme = UiAssetDocument {
        asset: UiAssetHeader {
            kind: UiAssetKind::Style,
            id: style_document_id.to_string(),
            version: 1,
            display_name: display_name.to_string(),
        },
        imports: UiAssetImports {
            widgets: Vec::new(),
            styles: document.imports.styles.clone(),
        },
        tokens: std::mem::take(&mut document.tokens),
        root: None,
        nodes: Default::default(),
        components: Default::default(),
        stylesheets: std::mem::take(&mut document.stylesheets),
    };

    document.imports.styles.clear();
    document.imports.styles.push(style_asset_id.to_string());

    Some(promoted_theme)
}

pub(crate) fn detach_imported_theme_to_local_theme_layer(
    document: &mut UiAssetDocument,
    imported_reference: &str,
    imported_style_document: &UiAssetDocument,
) -> bool {
    merge_imported_theme_into_local_theme_layer(
        document,
        imported_reference,
        imported_style_document,
        false,
    )
}

pub(crate) fn clone_imported_theme_to_local_theme_layer(
    document: &mut UiAssetDocument,
    imported_reference: &str,
    imported_style_document: &UiAssetDocument,
) -> bool {
    merge_imported_theme_into_local_theme_layer(
        document,
        imported_reference,
        imported_style_document,
        true,
    )
}

pub(crate) fn build_imported_theme_local_merge_preview(
    document: &UiAssetDocument,
    imported_reference: &str,
    imported_style_document: &UiAssetDocument,
) -> Vec<String> {
    let mut preview_items = Vec::new();
    append_imported_theme_merge_preview(
        &mut preview_items,
        "Detach",
        document,
        imported_reference,
        imported_style_document,
        detach_imported_theme_to_local_theme_layer,
    );
    append_imported_theme_merge_preview(
        &mut preview_items,
        "Clone",
        document,
        imported_reference,
        imported_style_document,
        clone_imported_theme_to_local_theme_layer,
    );
    preview_items
}

fn merge_imported_theme_into_local_theme_layer(
    document: &mut UiAssetDocument,
    imported_reference: &str,
    imported_style_document: &UiAssetDocument,
    keep_import_reference: bool,
) -> bool {
    let Some(import_index) = document
        .imports
        .styles
        .iter()
        .position(|reference| reference == imported_reference)
    else {
        return false;
    };

    let source_prefix = theme_base_name(imported_reference);
    let token_renames =
        build_imported_token_rename_map(&document.tokens, &imported_style_document.tokens, &source_prefix);
    merge_detached_style_imports(
        document,
        imported_reference,
        imported_style_document,
        import_index,
        keep_import_reference,
    );
    merge_detached_tokens(document, imported_style_document, &token_renames);
    merge_detached_stylesheets(
        &mut document.stylesheets,
        &imported_style_document.stylesheets,
        &token_renames,
        &source_prefix,
    );
    true
}

fn append_imported_theme_merge_preview(
    preview_items: &mut Vec<String>,
    mode_label: &str,
    document: &UiAssetDocument,
    imported_reference: &str,
    imported_style_document: &UiAssetDocument,
    apply_merge: fn(&mut UiAssetDocument, &str, &UiAssetDocument) -> bool,
) {
    let mut merged_document = document.clone();
    if !apply_merge(
        &mut merged_document,
        imported_reference,
        imported_style_document,
    ) {
        return;
    }

    preview_items.push(format!(
        "{mode_label} • imports • {}",
        if merged_document.imports.styles.is_empty() {
            "none".to_string()
        } else {
            merged_document.imports.styles.join(", ")
        }
    ));
    for (token_name, token_value) in &merged_document.tokens {
        preview_items.push(format!(
            "{mode_label} • token • {token_name} = {token_value}"
        ));
    }
    for stylesheet in &merged_document.stylesheets {
        for rule in &stylesheet.rules {
            let rule_label = if stylesheet.id.is_empty() {
                rule.selector.clone()
            } else {
                format!("{} • {}", stylesheet.id, rule.selector)
            };
            preview_items.push(format!("{mode_label} • rule • {rule_label}"));
        }
    }
}

fn merge_detached_style_imports(
    document: &mut UiAssetDocument,
    imported_reference: &str,
    imported_style_document: &UiAssetDocument,
    import_index: usize,
    keep_import_reference: bool,
) {
    if !keep_import_reference {
        document.imports.styles.remove(import_index);
    }
    let mut seen = document
        .imports
        .styles
        .iter()
        .cloned()
        .collect::<BTreeSet<_>>();
    let mut insert_index = if keep_import_reference {
        (import_index + 1).min(document.imports.styles.len())
    } else {
        import_index.min(document.imports.styles.len())
    };
    for nested_reference in &imported_style_document.imports.styles {
        if nested_reference == imported_reference || !seen.insert(nested_reference.clone()) {
            continue;
        }
        document
            .imports
            .styles
            .insert(insert_index, nested_reference.clone());
        insert_index += 1;
    }
}

fn merge_detached_tokens(
    document: &mut UiAssetDocument,
    imported_style_document: &UiAssetDocument,
    token_renames: &BTreeMap<String, String>,
) {
    for (token_name, token_value) in &imported_style_document.tokens {
        let mut rewritten_value = token_value.clone();
        rewrite_token_references_in_value(&mut rewritten_value, token_renames);
        let merged_name = token_renames
            .get(token_name)
            .cloned()
            .unwrap_or_else(|| token_name.clone());
        document.tokens.insert(merged_name, rewritten_value);
    }
}

fn merge_detached_stylesheets(
    local_stylesheets: &mut Vec<UiStyleSheet>,
    imported_stylesheets: &[UiStyleSheet],
    token_renames: &BTreeMap<String, String>,
    source_prefix: &str,
) {
    let mut used_ids = local_stylesheets
        .iter()
        .filter_map(|stylesheet| (!stylesheet.id.is_empty()).then_some(stylesheet.id.clone()))
        .collect::<BTreeSet<_>>();
    let mut merged_stylesheets = Vec::with_capacity(imported_stylesheets.len() + local_stylesheets.len());
    for stylesheet in imported_stylesheets.iter().cloned() {
        let mut stylesheet = stylesheet;
        rewrite_stylesheet_token_references(&mut stylesheet, token_renames);
        if !stylesheet.id.is_empty() {
            let original_id = stylesheet.id.clone();
            if used_ids.contains(&original_id) {
                stylesheet.id = unique_identifier(
                    &used_ids,
                    &format!("{source_prefix}_{original_id}"),
                );
            }
            used_ids.insert(stylesheet.id.clone());
        }
        merged_stylesheets.push(stylesheet);
    }
    merged_stylesheets.extend(std::mem::take(local_stylesheets));
    *local_stylesheets = merged_stylesheets;
}

fn build_imported_token_rename_map(
    existing_tokens: &BTreeMap<String, Value>,
    imported_tokens: &BTreeMap<String, Value>,
    source_prefix: &str,
) -> BTreeMap<String, String> {
    let mut used_names = existing_tokens.keys().cloned().collect::<BTreeSet<_>>();
    let mut rename_map = BTreeMap::new();
    for token_name in imported_tokens.keys() {
        let merged_name = if used_names.contains(token_name) {
            unique_identifier(&used_names, &format!("{source_prefix}_{token_name}"))
        } else {
            token_name.clone()
        };
        used_names.insert(merged_name.clone());
        rename_map.insert(token_name.clone(), merged_name);
    }
    rename_map
}

fn rewrite_stylesheet_token_references(
    stylesheet: &mut UiStyleSheet,
    token_renames: &BTreeMap<String, String>,
) {
    for rule in &mut stylesheet.rules {
        rewrite_declaration_block_token_references(&mut rule.set, token_renames);
    }
}

fn rewrite_declaration_block_token_references(
    declarations: &mut UiStyleDeclarationBlock,
    token_renames: &BTreeMap<String, String>,
) {
    for value in declarations.self_values.values_mut() {
        rewrite_token_references_in_value(value, token_renames);
    }
    for value in declarations.slot.values_mut() {
        rewrite_token_references_in_value(value, token_renames);
    }
}

fn rewrite_token_references_in_value(value: &mut Value, token_renames: &BTreeMap<String, String>) {
    match value {
        Value::String(text) => {
            let Some(token_name) = text.strip_prefix('$') else {
                return;
            };
            let Some(renamed) = token_renames.get(token_name) else {
                return;
            };
            *text = format!("${renamed}");
        }
        Value::Array(items) => {
            for item in items {
                rewrite_token_references_in_value(item, token_renames);
            }
        }
        Value::Table(table) => {
            for (_key, nested_value) in table.iter_mut() {
                rewrite_token_references_in_value(nested_value, token_renames);
            }
        }
        _ => {}
    }
}

fn unique_identifier(used_names: &BTreeSet<String>, base_name: &str) -> String {
    if !used_names.contains(base_name) {
        return base_name.to_string();
    }
    let mut suffix = 2usize;
    loop {
        let candidate = format!("{base_name}_{suffix}");
        if !used_names.contains(&candidate) {
            return candidate;
        }
        suffix += 1;
    }
}

fn theme_base_name(source_asset_id: &str) -> String {
    let normalized = source_asset_id.replace('\\', "/");
    let file_name = normalized.rsplit('/').next().unwrap_or("theme");
    let stem = file_name
        .strip_suffix(".ui.toml")
        .or_else(|| file_name.strip_suffix(".toml"))
        .unwrap_or(file_name);
    let stem = stem.strip_suffix(".ui").unwrap_or(stem);
    let mut normalized_name = String::new();
    let mut previous_was_separator = false;
    for ch in stem.chars() {
        if ch.is_ascii_alphanumeric() {
            normalized_name.push(ch.to_ascii_lowercase());
            previous_was_separator = false;
        } else if !previous_was_separator && !normalized_name.is_empty() {
            normalized_name.push('_');
            previous_was_separator = true;
        }
    }
    let normalized = normalized_name.trim_matches('_');
    if normalized.is_empty() {
        "theme".to_string()
    } else {
        normalized.to_string()
    }
}

fn theme_display_name(source_display_name: &str, base_name: &str) -> String {
    let trimmed = source_display_name.trim();
    if trimmed.is_empty() {
        let mut title = String::new();
        let mut capitalize_next = true;
        for ch in base_name.chars() {
            if ch == '_' || ch == '-' || ch == '.' {
                if !title.ends_with(' ') {
                    title.push(' ');
                }
                capitalize_next = true;
            } else if capitalize_next {
                title.push(ch.to_ascii_uppercase());
                capitalize_next = false;
            } else {
                title.push(ch);
            }
        }
        let title = title.trim().to_string();
        if title.is_empty() {
            "Theme".to_string()
        } else {
            format!("{title} Theme")
        }
    } else if trimmed.ends_with("Theme") {
        trimmed.to_string()
    } else {
        format!("{trimmed} Theme")
    }
}
