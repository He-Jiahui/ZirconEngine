use std::collections::{BTreeMap, BTreeSet, HashSet};

use toml::Value;
use zircon_runtime_interface::ui::template::{
    UiAssetDocument, UiStyleDeclarationBlock, UiStyleRule, UiStyleSheet,
};

pub(super) fn merge_imported_theme_into_local_theme_layer(
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
    let token_renames = build_imported_token_rename_map(
        &document.tokens,
        &imported_style_document.tokens,
        &source_prefix,
    );
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

pub(super) fn imported_theme_tokens(
    document: &UiAssetDocument,
    imported_styles: &BTreeMap<String, UiAssetDocument>,
) -> BTreeMap<String, Value> {
    let mut tokens = BTreeMap::new();
    for reference in &document.imports.styles {
        let Some(imported) = imported_styles.get(reference) else {
            continue;
        };
        for (token_name, value) in &imported.tokens {
            tokens
                .entry(token_name.clone())
                .or_insert_with(|| value.clone());
        }
    }
    tokens
}

pub(super) fn imported_theme_rules(
    document: &UiAssetDocument,
    imported_styles: &BTreeMap<String, UiAssetDocument>,
) -> BTreeSet<String> {
    let mut rules = BTreeSet::new();
    for reference in &document.imports.styles {
        let Some(imported) = imported_styles.get(reference) else {
            continue;
        };
        for stylesheet in &imported.stylesheets {
            for rule in &stylesheet.rules {
                let _ = rules.insert(rule_signature(rule));
            }
        }
    }
    rules
}

pub(super) fn stylesheet_label(stylesheet: &UiStyleSheet) -> String {
    if stylesheet.id.is_empty() {
        "<inline>".to_string()
    } else {
        stylesheet.id.clone()
    }
}

pub(super) fn rule_signature(rule: &UiStyleRule) -> String {
    format!(
        "{}|{}",
        rule.selector,
        toml::to_string(&rule.set).unwrap_or_default()
    )
}

pub(super) fn append_imported_theme_merge_preview(
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

struct UsedIdentifierSet<'a> {
    existing: HashSet<&'a str>,
    admitted: HashSet<String>,
}

impl<'a> UsedIdentifierSet<'a> {
    fn from_existing(
        entries: impl ExactSizeIterator<Item = &'a str>,
        admitted_capacity: usize,
    ) -> Self {
        let mut existing = HashSet::with_capacity(entries.len());
        existing.extend(entries);
        Self {
            existing,
            admitted: HashSet::with_capacity(admitted_capacity),
        }
    }

    fn contains(&self, identifier: &str) -> bool {
        self.existing.contains(identifier) || self.admitted.contains(identifier)
    }

    fn insert(&mut self, identifier: String) -> bool {
        if self.contains(&identifier) {
            return false;
        }
        self.admitted.insert(identifier)
    }
}

pub(super) fn merge_detached_style_imports(
    document: &mut UiAssetDocument,
    imported_reference: &str,
    imported_style_document: &UiAssetDocument,
    import_index: usize,
    keep_import_reference: bool,
) {
    if !keep_import_reference {
        document.imports.styles.remove(import_index);
    }
    let mut seen = UsedIdentifierSet::from_existing(
        document.imports.styles.iter().map(String::as_str),
        imported_style_document.imports.styles.len(),
    );
    let nested_references = imported_style_document
        .imports
        .styles
        .iter()
        .filter_map(|nested_reference| {
            (nested_reference != imported_reference && seen.insert(nested_reference.clone()))
                .then(|| nested_reference.clone())
        })
        .collect::<Vec<_>>();
    drop(seen);
    let mut insert_index = if keep_import_reference {
        (import_index + 1).min(document.imports.styles.len())
    } else {
        import_index.min(document.imports.styles.len())
    };
    for nested_reference in nested_references {
        document
            .imports
            .styles
            .insert(insert_index, nested_reference);
        insert_index += 1;
    }
}

pub(super) fn merge_detached_tokens(
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

pub(super) fn merge_detached_stylesheets(
    local_stylesheets: &mut Vec<UiStyleSheet>,
    imported_stylesheets: &[UiStyleSheet],
    token_renames: &BTreeMap<String, String>,
    source_prefix: &str,
) {
    let mut used_ids = UsedIdentifierSet::from_existing(
        local_stylesheets
            .iter()
            .filter_map(|stylesheet| (!stylesheet.id.is_empty()).then_some(stylesheet.id.as_str())),
        imported_stylesheets.len(),
    );
    let mut merged_stylesheets =
        Vec::with_capacity(imported_stylesheets.len() + local_stylesheets.len());
    for stylesheet in imported_stylesheets.iter().cloned() {
        let mut stylesheet = stylesheet;
        rewrite_stylesheet_token_references(&mut stylesheet, token_renames);
        if !stylesheet.id.is_empty() {
            if used_ids.contains(&stylesheet.id) {
                let collision_base = format!("{source_prefix}_{}", stylesheet.id);
                stylesheet.id = unique_identifier(&used_ids, &collision_base);
            }
            used_ids.insert(stylesheet.id.clone());
        }
        merged_stylesheets.push(stylesheet);
    }
    drop(used_ids);
    merged_stylesheets.extend(std::mem::take(local_stylesheets));
    *local_stylesheets = merged_stylesheets;
}

pub(super) fn build_imported_token_rename_map(
    existing_tokens: &BTreeMap<String, Value>,
    imported_tokens: &BTreeMap<String, Value>,
    source_prefix: &str,
) -> BTreeMap<String, String> {
    let mut used_names = UsedIdentifierSet::from_existing(
        existing_tokens.keys().map(String::as_str),
        imported_tokens.len(),
    );
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

pub(super) fn rewrite_stylesheet_token_references(
    stylesheet: &mut UiStyleSheet,
    token_renames: &BTreeMap<String, String>,
) {
    for rule in &mut stylesheet.rules {
        rewrite_declaration_block_token_references(&mut rule.set, token_renames);
    }
}

pub(super) fn rewrite_declaration_block_token_references(
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

pub(super) fn rewrite_token_references_in_value(
    value: &mut Value,
    token_renames: &BTreeMap<String, String>,
) {
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

fn unique_identifier(used_names: &UsedIdentifierSet<'_>, base_name: &str) -> String {
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

pub(super) fn theme_base_name(source_asset_id: &str) -> String {
    let normalized = source_asset_id.replace('\\', "/");
    let file_name = normalized.rsplit('/').next().unwrap_or("theme");
    let stem = file_name
        .strip_suffix(".zui")
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

pub(super) fn theme_display_name(source_display_name: &str, base_name: &str) -> String {
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

#[cfg(test)]
mod optimization_tests {
    use std::collections::BTreeSet;
    use std::hint::black_box;
    use std::time::Instant;

    use zircon_runtime_interface::ui::template::{
        UI_ASSET_CURRENT_SOURCE_SCHEMA_VERSION, UiAssetHeader, UiAssetKind,
    };

    use super::*;

    #[test]
    fn optimization_batch_20260826m_editor23_borrowed_merge_indexes_preserve_collisions() {
        let existing_tokens = BTreeMap::from([
            ("accent".to_string(), Value::String("#fff".to_string())),
            (
                "theme_accent".to_string(),
                Value::String("#eee".to_string()),
            ),
        ]);
        let imported_tokens = BTreeMap::from([
            ("accent".to_string(), Value::String("#000".to_string())),
            ("panel".to_string(), Value::String("#111".to_string())),
        ]);
        assert_eq!(
            build_imported_token_rename_map(&existing_tokens, &imported_tokens, "theme"),
            BTreeMap::from([
                ("accent".to_string(), "theme_accent_2".to_string()),
                ("panel".to_string(), "panel".to_string()),
            ])
        );

        let mut stylesheets = vec![stylesheet("base"), stylesheet("theme_base")];
        merge_detached_stylesheets(
            &mut stylesheets,
            &[stylesheet("base"), stylesheet("fresh")],
            &BTreeMap::new(),
            "theme",
        );
        assert_eq!(
            stylesheets
                .iter()
                .map(|stylesheet| stylesheet.id.as_str())
                .collect::<Vec<_>>(),
            vec!["theme_base_2", "fresh", "base", "theme_base"]
        );

        let mut document = document("local-theme");
        document.imports.styles = vec!["base.zui".to_string(), "shared.zui".to_string()];
        let mut imported = document("imported-theme");
        imported.imports.styles = vec![
            "shared.zui".to_string(),
            "nested.zui".to_string(),
            "nested.zui".to_string(),
        ];
        merge_detached_style_imports(&mut document, "base.zui", &imported, 0, false);
        assert_eq!(
            document.imports.styles,
            vec!["nested.zui".to_string(), "shared.zui".to_string()]
        );
    }

    #[test]
    fn optimization_batch_20260826m_editor23_theme_merge_indexes_borrow_existing_ids() {
        let source = include_str!("merge.rs");
        let production = source
            .split("#[cfg(test)]")
            .next()
            .expect("theme merge production source");

        assert!(production.contains("struct UsedIdentifierSet"));
        assert_eq!(
            production
                .matches("UsedIdentifierSet::from_existing")
                .count(),
            3
        );
        assert!(production.contains("HashSet::with_capacity(entries.len())"));
        assert!(production.contains("HashSet::with_capacity(admitted_capacity)"));
        assert!(production.contains("fn unique_identifier(used_names: &UsedIdentifierSet<'_>"));
        assert!(!production.contains("existing_tokens.keys().cloned()"));
        assert!(!production.contains("stylesheet.id.clone()))\n        .collect::<BTreeSet"));
        assert!(
            !production.contains(".imports\n        .styles\n        .iter()\n        .cloned()")
        );
    }

    #[test]
    #[ignore = "release performance evidence; run through the validation coordinator"]
    fn optimization_batch_20260826m_editor23_theme_merge_borrowed_hash_performance_evidence() {
        let entries = (0..32_768)
            .map(|index| format!("editor_theme_existing_identifier_{index:05}_long_name"))
            .collect::<Vec<_>>();
        let copied_bytes = entries.iter().map(String::len).sum::<usize>();
        let mut legacy_samples = Vec::with_capacity(17);
        let mut hash_samples = Vec::with_capacity(17);
        for _ in 0..17 {
            let started = Instant::now();
            for _ in 0..3 {
                black_box(black_box(&entries).iter().cloned().collect::<BTreeSet<_>>());
            }
            legacy_samples.push(started.elapsed().as_nanos());

            let started = Instant::now();
            for _ in 0..3 {
                black_box(UsedIdentifierSet::from_existing(
                    black_box(&entries).iter().map(String::as_str),
                    0,
                ));
            }
            hash_samples.push(started.elapsed().as_nanos());
        }

        legacy_samples.sort_unstable();
        hash_samples.sort_unstable();
        let legacy_p95 = legacy_samples[16];
        let hash_p95 = hash_samples[16];
        println!(
            "EDITOR23_THEME_MERGE_BORROWED_ID_INDEX_BENCH_V1 entries_per_index={} indexes=3 legacy_p95_ns={} hash_p95_ns={} legacy_string_clones={} hash_string_clones=0 legacy_copied_bytes={} hash_copied_bytes=0 target_ratio_bp=6000",
            entries.len(),
            legacy_p95,
            hash_p95,
            entries.len() * 3,
            copied_bytes * 3,
        );
        assert!(
            hash_p95.saturating_mul(10_000) <= legacy_p95.saturating_mul(6_000),
            "borrowed theme merge hash P95 {hash_p95} ns exceeded 60% of legacy {legacy_p95} ns"
        );
    }

    fn stylesheet(id: &str) -> UiStyleSheet {
        UiStyleSheet {
            id: id.to_string(),
            rules: Vec::new(),
        }
    }

    fn document(id: &str) -> UiAssetDocument {
        UiAssetDocument {
            asset: UiAssetHeader {
                kind: UiAssetKind::Style,
                id: id.to_string(),
                version: UI_ASSET_CURRENT_SOURCE_SCHEMA_VERSION,
                display_name: id.to_string(),
            },
            imports: Default::default(),
            tokens: BTreeMap::new(),
            root: None,
            components: BTreeMap::new(),
            stylesheets: Vec::new(),
        }
    }
}
