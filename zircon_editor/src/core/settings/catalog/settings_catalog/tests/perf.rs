use std::hint::black_box;
use std::time::Instant;

use super::*;
use crate::core::settings::{
    settings_registry_with_defaults, SettingSchema, SettingValue, SettingsScope,
};

fn legacy_category_path(segments: &[String]) -> String {
    segments
        .iter()
        .map(String::as_str)
        .collect::<Vec<_>>()
        .join("/")
}

fn single_buffer_category_path(segments: &[String]) -> String {
    let segment_bytes = segments.iter().map(String::len).sum::<usize>();
    let mut path = String::with_capacity(segment_bytes + segments.len().saturating_sub(1));
    for (index, segment) in segments.iter().enumerate() {
        if index != 0 {
            path.push('/');
        }
        path.push_str(segment);
    }
    path
}

#[test]
fn subtree_category_index_includes_direct_and_descendant_keys_without_query_clones() {
    let mut registry = settings_registry_with_defaults();
    let direct_key = SettingsKey::parse("fixture.editor.parent").unwrap();
    registry
        .register(
            SettingDefinition::new(
                direct_key.clone(),
                SettingsScope::User,
                SettingSchema::Bool,
                SettingValue::Bool(false),
                false,
                SettingsPresentation::new(
                    "settings.fixture.editor.parent.label",
                    "settings.fixture.editor.parent.description",
                    ["settings.category.editor"],
                )
                .unwrap(),
            )
            .unwrap(),
        )
        .unwrap();
    let grandchild_key = SettingsKey::parse("fixture.editor.grandchild").unwrap();
    registry
        .register(
            SettingDefinition::new(
                grandchild_key.clone(),
                SettingsScope::User,
                SettingSchema::Bool,
                SettingValue::Bool(false),
                false,
                SettingsPresentation::new(
                    "settings.fixture.editor.grandchild.label",
                    "settings.fixture.editor.grandchild.description",
                    [
                        "settings.category.editor",
                        "settings.category.autosave",
                        "settings.category.advanced",
                    ],
                )
                .unwrap(),
            )
            .unwrap(),
        )
        .unwrap();
    let catalog = SettingsCatalog::from_registry(&registry);
    let parent_path = "settings.category.editor";
    let direct = catalog.keys_for_category_path(parent_path);
    let subtree = catalog
        .keys_for_category_subtree(parent_path)
        .collect::<Vec<_>>();

    assert_eq!(direct, std::slice::from_ref(&direct_key));
    assert!(subtree.contains(&&direct_key));
    assert!(subtree.contains(&&grandchild_key));
    assert!(subtree
        .iter()
        .any(|key| key.as_str() == "editor.language.locale"));
    assert!(subtree
        .iter()
        .any(|key| key.as_str() == "editor.autosave.interval_secs"));
    assert!(subtree.windows(2).all(|pair| pair[0] < pair[1]));
    assert!(subtree
        .iter()
        .all(|key| { std::ptr::eq(*key, &catalog.definition(key).unwrap().key) }));
    assert!(catalog
        .keys_for_category_subtree("settings.category.edit")
        .next()
        .is_none());
    assert_eq!(
        catalog
            .keys_for_category_subtree("settings.category.editor/settings.category.autosave")
            .count(),
        catalog
            .keys_for_category_path("settings.category.editor/settings.category.autosave")
            .len()
            + 1
    );
    assert_eq!(
        catalog
            .keys_for_category_subtree(
                "settings.category.editor/settings.category.autosave/settings.category.advanced"
            )
            .map(SettingsKey::as_str)
            .collect::<Vec<_>>(),
        vec![grandchild_key.as_str()]
    );
}

#[test]
#[ignore = "release category-query scale evidence; run through the validation coordinator"]
fn editor265_ten_thousand_definition_subtree_query_release_benchmark() {
    const CATEGORY_COUNT: usize = 1_000;
    const KEYS_PER_CATEGORY: usize = 10;
    const SAMPLE_PAIRS: usize = 21;
    let mut registry = SettingsRegistry::default();
    let mut categories = Vec::with_capacity(CATEGORY_COUNT);
    for category in 0..CATEGORY_COUNT {
        let segment = format!("settings.category.group_{category:04}");
        categories.push((
            segment.clone(),
            format!("settings.category.performance/{segment}"),
        ));
        for setting in 0..KEYS_PER_CATEGORY {
            registry
                .register(
                    SettingDefinition::new(
                        SettingsKey::parse(format!(
                            "fixture.performance.group_{category:04}.setting_{setting:02}"
                        ))
                        .unwrap(),
                        SettingsScope::User,
                        SettingSchema::Bool,
                        SettingValue::Bool(false),
                        false,
                        SettingsPresentation::new(
                            "settings.fixture.performance.label",
                            "settings.fixture.performance.description",
                            [
                                "settings.category.performance",
                                segment.as_str(),
                                "settings.category.leaf",
                            ],
                        )
                        .unwrap(),
                    )
                    .unwrap(),
                )
                .unwrap();
        }
    }
    let catalog = SettingsCatalog::from_registry(&registry);
    let queries = categories.iter().step_by(10).collect::<Vec<_>>();
    let indexed_count = queries
        .iter()
        .map(|(_, path)| {
            catalog
                .keys_for_category_subtree(path)
                .fold(0, |count, key| {
                    black_box(key.as_str());
                    count + 1
                })
        })
        .sum::<usize>();
    assert_eq!(indexed_count, queries.len() * KEYS_PER_CATEGORY);
    assert_eq!(
        catalog
            .subtree_index
            .values()
            .map(|indices| indices.len())
            .sum::<usize>(),
        CATEGORY_COUNT * KEYS_PER_CATEGORY * 3
    );

    let measure_indexed = || {
        let started = Instant::now();
        let count = black_box(&queries)
            .iter()
            .map(|(_, path)| {
                black_box(&catalog)
                    .keys_for_category_subtree(path)
                    .fold(0, |count, key| {
                        black_box(key.as_str());
                        count + 1
                    })
            })
            .sum::<usize>();
        black_box(count);
        started.elapsed().as_nanos().max(1)
    };
    let measure_scan = || {
        let started = Instant::now();
        let count = black_box(&queries)
            .iter()
            .map(|(segment, _)| {
                black_box(&catalog)
                    .definitions()
                    .iter()
                    .filter(|definition| {
                        definition.presentation().category_path().nth(1) == Some(segment.as_str())
                    })
                    .count()
            })
            .sum::<usize>();
        black_box(count);
        started.elapsed().as_nanos().max(1)
    };
    for _ in 0..2 {
        black_box(measure_scan());
        black_box(measure_indexed());
    }
    let mut scan_samples = Vec::with_capacity(SAMPLE_PAIRS);
    let mut indexed_samples = Vec::with_capacity(SAMPLE_PAIRS);
    for pair in 0..SAMPLE_PAIRS {
        if pair % 2 == 0 {
            scan_samples.push(measure_scan());
            indexed_samples.push(measure_indexed());
        } else {
            indexed_samples.push(measure_indexed());
            scan_samples.push(measure_scan());
        }
    }
    scan_samples.sort_unstable();
    indexed_samples.sort_unstable();
    let scan_p95 = scan_samples[19];
    let indexed_p95 = indexed_samples[19];
    println!(
        "EDITOR265_SUBTREE_QUERY_BENCH_V1 sample_pairs={SAMPLE_PAIRS} definitions={} categories={CATEGORY_COUNT} queried_categories={} scan_p95_ns={scan_p95} indexed_p95_ns={indexed_p95} target_ratio_bp=2000",
        CATEGORY_COUNT * KEYS_PER_CATEGORY,
        queries.len(),
    );
    assert!(
        indexed_p95.saturating_mul(10_000) <= scan_p95.saturating_mul(2_000),
        "indexed category query P95 {indexed_p95} ns exceeded 20% of scan P95 {scan_p95} ns"
    );
}

#[test]
fn optimization_batch_hy_editor608_single_buffer_preserves_catalog_paths() {
    let registry = settings_registry_with_defaults();
    let catalog = SettingsCatalog::from_registry(&registry);

    assert_eq!(
        catalog
            .keys_for_category_path("settings.category.viewport/settings.category.snapping")
            .len(),
        3
    );
    assert!(catalog
        .keys_for_category_path("settings.category.editor/settings.category.autosave")
        .iter()
        .any(|key| key.as_str() == "editor.autosave.interval_secs"));
}

#[test]
fn optimization_batch_hy_editor608_category_path_uses_one_owned_buffer() {
    let source = include_str!("../../settings_catalog.rs");
    let builder = source
        .split("fn category_path_key")
        .nth(1)
        .expect("category path builder")
        .split("#[cfg(test)]")
        .next()
        .expect("bounded category path builder");

    assert!(builder.contains("String::with_capacity"));
    assert!(builder.contains("path.push('/')"));
    assert!(builder.contains("path.push_str(category)"));
    assert!(!builder.contains("collect::<Vec<_>>()"));
    assert!(!builder.contains(".join("));
}

#[test]
#[ignore = "release performance evidence; run through the validation coordinator"]
fn optimization_batch_hy_editor608_category_path_single_buffer_performance_evidence() {
    const PATH_COUNT: usize = 8_192;
    const SEGMENTS_PER_PATH: usize = 12;
    const SAMPLE_PAIRS: usize = 17;
    let paths = (0..PATH_COUNT)
        .map(|path| {
            (0..SEGMENTS_PER_PATH)
                .map(|segment| format!("settings.category.performance_{path:05}_{segment:02}"))
                .collect::<Vec<_>>()
        })
        .collect::<Vec<_>>();
    let measure_legacy = || {
        let started = Instant::now();
        for path in black_box(&paths) {
            black_box(legacy_category_path(path));
        }
        started.elapsed().as_nanos().max(1)
    };
    let measure_single = || {
        let started = Instant::now();
        for path in black_box(&paths) {
            black_box(single_buffer_category_path(path));
        }
        started.elapsed().as_nanos().max(1)
    };
    for _ in 0..3 {
        black_box(measure_legacy());
        black_box(measure_single());
    }

    let mut legacy_samples = Vec::with_capacity(SAMPLE_PAIRS);
    let mut single_samples = Vec::with_capacity(SAMPLE_PAIRS);
    for pair in 0..SAMPLE_PAIRS {
        if pair % 2 == 0 {
            legacy_samples.push(measure_legacy());
            single_samples.push(measure_single());
        } else {
            single_samples.push(measure_single());
            legacy_samples.push(measure_legacy());
        }
    }
    legacy_samples.sort_unstable();
    single_samples.sort_unstable();
    let legacy_p50 = legacy_samples[8];
    let legacy_p95 = legacy_samples[16];
    let single_p50 = single_samples[8];
    let single_p95 = single_samples[16];
    println!(
        "EDITOR608_CATEGORY_PATH_SINGLE_BUFFER_BENCH_V1 sample_pairs={SAMPLE_PAIRS} pair_order=alternating_legacy_even legacy_first_pairs=9 single_first_pairs=8 paths={PATH_COUNT} segments_per_path={SEGMENTS_PER_PATH} legacy_p50_ns={legacy_p50} legacy_p95_ns={legacy_p95} single_p50_ns={single_p50} single_p95_ns={single_p95} owned_buffers_per_path=2->1 target_ratio_bp=8000"
    );
    assert!(
        single_p95.saturating_mul(10_000) <= legacy_p95.saturating_mul(8_000),
        "single-buffer category path P95 {single_p95} ns exceeded 80% of legacy {legacy_p95} ns"
    );
}
