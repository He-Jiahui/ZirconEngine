use zircon_runtime_interface::ui::template::{
    UiAssetDocument, UiAssetHeader, UiAssetImports, UiAssetKind,
    UI_ASSET_CURRENT_SOURCE_SCHEMA_VERSION,
};

use super::merge::{theme_base_name, theme_display_name};

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct UiAssetExternalStyleDraft {
    pub(crate) asset_id: String,
    pub(crate) document_id: String,
    pub(crate) display_name: String,
}

pub(crate) fn can_promote_local_theme_to_external_style_asset(document: &UiAssetDocument) -> bool {
    !document.tokens.is_empty() || !document.stylesheets.is_empty()
}

fn take_and_replace_style_imports(
    style_imports: &mut Vec<String>,
    style_asset_id: &str,
) -> Vec<String> {
    let promoted_style_imports = std::mem::take(style_imports);
    style_imports.push(style_asset_id.to_string());
    promoted_style_imports
}

pub(crate) fn default_external_style_draft(
    source_asset_id: &str,
    source_display_name: &str,
) -> UiAssetExternalStyleDraft {
    let base_name = theme_base_name(source_asset_id);
    UiAssetExternalStyleDraft {
        asset_id: format!("res://ui/themes/{base_name}_theme.zui"),
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

    let promoted_style_imports =
        take_and_replace_style_imports(&mut document.imports.styles, style_asset_id);
    let promoted_theme = UiAssetDocument {
        asset: UiAssetHeader {
            kind: UiAssetKind::Style,
            id: style_document_id.to_string(),
            version: UI_ASSET_CURRENT_SOURCE_SCHEMA_VERSION,
            display_name: display_name.to_string(),
        },
        imports: UiAssetImports {
            widgets: Vec::new(),
            styles: promoted_style_imports,
            resources: Vec::new(),
        },
        tokens: std::mem::take(&mut document.tokens),
        root: None,
        components: Default::default(),
        stylesheets: std::mem::take(&mut document.stylesheets),
    };

    Some(promoted_theme)
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;
    use std::hint::black_box;
    use std::time::Instant;

    use super::*;

    #[test]
    fn optimization_batch_ds_theme_promotion_moves_style_import_storage() {
        let mut tokens = BTreeMap::new();
        tokens.insert(
            "accent".to_string(),
            toml::Value::String("#ff0066".to_string()),
        );
        let mut document = UiAssetDocument {
            asset: UiAssetHeader {
                kind: UiAssetKind::Layout,
                id: "ui.screen.inventory".to_string(),
                version: UI_ASSET_CURRENT_SOURCE_SCHEMA_VERSION,
                display_name: "Inventory".to_string(),
            },
            imports: UiAssetImports {
                widgets: Vec::new(),
                styles: vec![
                    "res://ui/themes/base.zui".to_string(),
                    "res://ui/themes/accessibility.zui".to_string(),
                ],
                resources: Vec::new(),
            },
            tokens,
            root: None,
            components: BTreeMap::new(),
            stylesheets: Vec::new(),
        };
        let original_pointer = document.imports.styles.as_ptr();

        let promoted = promote_local_theme_to_external_style_asset(
            &mut document,
            "res://ui/themes/inventory_theme.zui",
            "ui.theme.inventory",
            "Inventory Theme",
        )
        .expect("local tokens should be promotable");

        assert_eq!(promoted.imports.styles.as_ptr(), original_pointer);
        assert_eq!(
            promoted.imports.styles,
            vec![
                "res://ui/themes/base.zui".to_string(),
                "res://ui/themes/accessibility.zui".to_string()
            ]
        );
        assert_eq!(
            document.imports.styles,
            vec!["res://ui/themes/inventory_theme.zui".to_string()]
        );
        assert!(document.tokens.is_empty());
        assert_eq!(
            promoted.tokens.get("accent").and_then(toml::Value::as_str),
            Some("#ff0066")
        );
    }

    #[test]
    fn optimization_batch_ds_theme_promotion_uses_owned_style_imports() {
        let production = include_str!("promotion.rs")
            .split("#[cfg(test)]")
            .next()
            .expect("theme promotion production source");

        assert!(production.contains("std::mem::take(style_imports)"));
        assert!(production.contains("styles: promoted_style_imports"));
        assert!(!production.contains("styles: document.imports.styles.clone()"));
    }

    #[test]
    #[ignore = "release-only alternating p95 performance gate"]
    fn optimization_batch_ds_theme_promotion_owned_style_imports_p95() {
        const SAMPLE_PAIRS: usize = 17;
        const PROMOTIONS_PER_SAMPLE: usize = 2_048;
        const IMPORTS_PER_PROMOTION: usize = 256;

        let seed = (0..IMPORTS_PER_PROMOTION)
            .map(|index| format!("res://ui/themes/shared/theme_{index:04}_with_long_name.zui"))
            .collect::<Vec<_>>();
        let mut legacy_samples = Vec::with_capacity(SAMPLE_PAIRS);
        let mut optimized_samples = Vec::with_capacity(SAMPLE_PAIRS);
        for sample_index in 0..SAMPLE_PAIRS {
            if sample_index % 2 == 0 {
                legacy_samples.push(measure_promotions(&seed, PROMOTIONS_PER_SAMPLE, false));
                optimized_samples.push(measure_promotions(&seed, PROMOTIONS_PER_SAMPLE, true));
            } else {
                optimized_samples.push(measure_promotions(&seed, PROMOTIONS_PER_SAMPLE, true));
                legacy_samples.push(measure_promotions(&seed, PROMOTIONS_PER_SAMPLE, false));
            }
        }

        let legacy_p95 = p95(&mut legacy_samples);
        let optimized_p95 = p95(&mut optimized_samples);
        println!(
            "EDITOR355_THEME_PROMOTION_OWNED_STYLE_IMPORTS_BENCH_V1 promotions_per_sample={PROMOTIONS_PER_SAMPLE} imports_per_promotion={IMPORTS_PER_PROMOTION} legacy_p95_ns={legacy_p95} optimized_p95_ns={optimized_p95} ratio={:.4}",
            optimized_p95 as f64 / legacy_p95.max(1) as f64
        );
        assert!(
            optimized_p95.saturating_mul(100) <= legacy_p95.saturating_mul(70),
            "theme promotion owned style imports p95 {optimized_p95}ns exceeded 70% of legacy {legacy_p95}ns"
        );

        fn measure_promotions(seed: &[String], promotion_count: usize, optimized: bool) -> u128 {
            let started_at = Instant::now();
            let mut checksum = 0_usize;
            for _ in 0..promotion_count {
                let mut style_imports = seed.to_vec();
                let promoted_style_imports = if optimized {
                    super::take_and_replace_style_imports(
                        &mut style_imports,
                        "res://ui/themes/promoted.zui",
                    )
                } else {
                    let promoted_style_imports = style_imports.clone();
                    style_imports.clear();
                    style_imports.push("res://ui/themes/promoted.zui".to_string());
                    promoted_style_imports
                };
                checksum = checksum
                    .wrapping_add(promoted_style_imports.len())
                    .wrapping_add(style_imports[0].len());
                black_box((&style_imports, &promoted_style_imports));
            }
            black_box(checksum);
            started_at.elapsed().as_nanos()
        }

        fn p95(samples: &mut [u128]) -> u128 {
            samples.sort_unstable();
            samples[(samples.len() * 95).div_ceil(100).saturating_sub(1)]
        }
    }
}
