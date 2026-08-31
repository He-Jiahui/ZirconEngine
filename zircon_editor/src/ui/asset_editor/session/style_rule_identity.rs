use std::collections::HashSet;

use zircon_runtime_interface::ui::template::UiAssetDocument;

pub(super) fn unique_style_rule_id(document: &UiAssetDocument, selector: &str) -> String {
    let base = style_rule_id_stem(selector);
    let rule_count = document
        .stylesheets
        .iter()
        .map(|stylesheet| stylesheet.rules.len())
        .sum();
    let mut used_ids: HashSet<&str> = HashSet::with_capacity(rule_count);
    used_ids.extend(
        document
            .stylesheets
            .iter()
            .flat_map(|stylesheet| stylesheet.rules.iter())
            .filter_map(|rule| rule.id.as_deref()),
    );
    if !used_ids.contains(base.as_str()) {
        return base;
    }

    let mut suffix = 2;
    loop {
        let candidate = format!("{base}_{suffix}");
        if !used_ids.contains(candidate.as_str()) {
            return candidate;
        }
        suffix += 1;
    }
}

fn style_rule_id_stem(selector: &str) -> String {
    let mut stem = String::new();
    let mut previous_was_word = false;
    let mut previous_was_lower_or_digit = false;
    for character in selector.chars() {
        if character.is_ascii_alphanumeric() {
            if character.is_ascii_uppercase() && previous_was_lower_or_digit {
                push_separator(&mut stem);
            }
            stem.push(character.to_ascii_lowercase());
            previous_was_word = true;
            previous_was_lower_or_digit =
                character.is_ascii_lowercase() || character.is_ascii_digit();
        } else {
            if previous_was_word {
                push_separator(&mut stem);
            }
            previous_was_word = false;
            previous_was_lower_or_digit = false;
        }
    }

    while stem.ends_with('_') {
        stem.pop();
    }
    if stem.is_empty() {
        return "style_rule".to_string();
    }
    if stem
        .chars()
        .next()
        .is_some_and(|character| character.is_ascii_digit())
    {
        return format!("rule_{stem}");
    }
    stem
}

fn push_separator(stem: &mut String) {
    if !stem.is_empty() && !stem.ends_with('_') {
        stem.push('_');
    }
}

#[cfg(test)]
mod optimization_tests {
    use std::collections::BTreeSet;
    use std::hint::black_box;
    use std::time::Instant;

    use zircon_runtime_interface::ui::template::{UiStyleRule, UiStyleSheet};

    use super::*;

    fn document_with_rule_ids(rule_ids: impl IntoIterator<Item = String>) -> UiAssetDocument {
        UiAssetDocument {
            stylesheets: vec![UiStyleSheet {
                id: "main".to_string(),
                rules: rule_ids
                    .into_iter()
                    .map(|id| UiStyleRule {
                        id: Some(id),
                        selector: ".fixture".to_string(),
                        set: Default::default(),
                    })
                    .collect(),
            }],
            ..UiAssetDocument::default()
        }
    }

    #[test]
    fn optimization_batch_20260826j_editor23_style_rule_id_preserves_first_free_suffix() {
        let document = document_with_rule_ids(
            ["primary_button", "primary_button_2", "primary_button_3"]
                .into_iter()
                .map(str::to_string),
        );

        assert_eq!(
            unique_style_rule_id(&document, ".PrimaryButton"),
            "primary_button_4"
        );
        assert_eq!(unique_style_rule_id(&document, ".FreshRule"), "fresh_rule");
    }

    #[test]
    fn optimization_batch_20260826j_editor23_style_rule_id_uses_capacity_hash_index() {
        let source = include_str!("style_rule_identity.rs");
        let production = source
            .split("#[cfg(test)]")
            .next()
            .expect("style rule identity production source");
        let generator = production
            .split("pub(super) fn unique_style_rule_id")
            .nth(1)
            .expect("style rule ID generator")
            .split("fn style_rule_id_stem")
            .next()
            .expect("bounded style rule ID generator");

        assert!(!production.contains("BTreeSet"));
        assert!(generator.contains("HashSet::with_capacity(rule_count)"));
        assert!(generator.contains("HashSet<&str>"));
        assert!(!generator.contains(".cloned()"));
    }

    #[test]
    #[ignore = "release performance evidence; run through the validation coordinator"]
    fn optimization_batch_20260826j_editor23_style_rule_id_hash_index_performance_evidence() {
        fn legacy_unique_style_rule_id(document: &UiAssetDocument, selector: &str) -> String {
            let base = style_rule_id_stem(selector);
            let used_ids = document
                .stylesheets
                .iter()
                .flat_map(|stylesheet| stylesheet.rules.iter())
                .filter_map(|rule| rule.id.as_deref())
                .collect::<BTreeSet<_>>();
            if !used_ids.contains(base.as_str()) {
                return base;
            }
            let mut suffix = 2;
            loop {
                let candidate = format!("{base}_{suffix}");
                if !used_ids.contains(candidate.as_str()) {
                    return candidate;
                }
                suffix += 1;
            }
        }

        let document = document_with_rule_ids(
            (0..32_768).map(|index| format!("editor.asset.theme.rule.{index:05}")),
        );
        let mut legacy_samples = Vec::with_capacity(17);
        let mut hash_samples = Vec::with_capacity(17);
        for _ in 0..17 {
            let started = Instant::now();
            black_box(legacy_unique_style_rule_id(
                black_box(&document),
                black_box(".FreshRule"),
            ));
            legacy_samples.push(started.elapsed().as_nanos());

            let started = Instant::now();
            black_box(unique_style_rule_id(
                black_box(&document),
                black_box(".FreshRule"),
            ));
            hash_samples.push(started.elapsed().as_nanos());
        }

        legacy_samples.sort_unstable();
        hash_samples.sort_unstable();
        let legacy_p95 = legacy_samples[16];
        let hash_p95 = hash_samples[16];
        println!(
            "EDITOR23_STYLE_RULE_ID_HASH_INDEX_BENCH_V1 rule_ids={} legacy_p95_ns={} hash_p95_ns={} legacy_tree_admissions={} hash_admissions={} borrowed_ids_before={} borrowed_ids_after={} target_ratio_bp=6000",
            document.stylesheets[0].rules.len(),
            legacy_p95,
            hash_p95,
            document.stylesheets[0].rules.len(),
            document.stylesheets[0].rules.len(),
            document.stylesheets[0].rules.len(),
            document.stylesheets[0].rules.len(),
        );
        assert!(
            hash_p95.saturating_mul(10_000) <= legacy_p95.saturating_mul(6_000),
            "style rule ID HashSet P95 {hash_p95} ns exceeded 60% of legacy {legacy_p95} ns"
        );
    }
}
