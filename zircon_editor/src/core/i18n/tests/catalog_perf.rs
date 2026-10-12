use std::collections::BTreeMap;
use std::hint::black_box;
use std::sync::Arc;
use std::time::Instant;

use super::EditorLocale;

#[test]
fn optimization_batch_ie_editor616_translate_borrows_active_locale_guard() {
    let source = include_str!("../catalog.rs");
    let body = source
        .split("    pub fn translate(&self, key: &str) -> Arc<str> {")
        .nth(1)
        .and_then(|body| {
            body.split("    /// Resolves against one captured locale")
                .next()
        })
        .unwrap();

    assert!(body.contains("let active_locale = self.read_active_locale()"));
    assert!(body.contains("self.translate_for_locale(&active_locale, key)"));
    assert!(!body.contains("self.active_locale()"));
}

#[test]
#[ignore = "managed Windows release performance evidence"]
fn optimization_batch_ie_editor616_borrowed_locale_lookup_p95() {
    const ITERATIONS: usize = 262_144;
    const SAMPLE_PAIRS: usize = 17;
    let locale = EditorLocale::parse("zh-Hans-CN").unwrap();
    let bundles = BTreeMap::from([
        (EditorLocale::english(), Arc::<str>::from("English")),
        (locale.clone(), Arc::<str>::from("Simplified Chinese")),
    ]);
    let mut retired = Vec::with_capacity(SAMPLE_PAIRS);
    let mut optimized = Vec::with_capacity(SAMPLE_PAIRS);
    for pair in 0..SAMPLE_PAIRS {
        if pair % 2 == 0 {
            retired.push(measure_locale_lookup(&bundles, &locale, ITERATIONS, true));
            optimized.push(measure_locale_lookup(&bundles, &locale, ITERATIONS, false));
        } else {
            optimized.push(measure_locale_lookup(&bundles, &locale, ITERATIONS, false));
            retired.push(measure_locale_lookup(&bundles, &locale, ITERATIONS, true));
        }
    }
    let retired_p95_ns = percentile(&retired, 95);
    let optimized_p95_ns = percentile(&optimized, 95);
    println!(
        "EDITOR616_BORROWED_ACTIVE_LOCALE_LOOKUP_BENCH_V1 sample_pairs={SAMPLE_PAIRS} \
             iterations={ITERATIONS} retired_arc_clones={ITERATIONS} optimized_arc_clones=0 \
             retired_p95_ns={retired_p95_ns} optimized_p95_ns={optimized_p95_ns} \
             retired_raw_ns={} optimized_raw_ns={}",
        csv(&retired),
        csv(&optimized)
    );
    assert!(
        optimized_p95_ns.saturating_mul(100) <= retired_p95_ns.saturating_mul(80),
        "borrowed active-locale lookup P95 must be at most 80% of clone-per-lookup: retired={retired_p95_ns}ns optimized={optimized_p95_ns}ns"
    );
}

fn measure_locale_lookup(
    bundles: &BTreeMap<EditorLocale, Arc<str>>,
    locale: &EditorLocale,
    iterations: usize,
    retired: bool,
) -> u128 {
    let started = Instant::now();
    let mut checksum = 0usize;
    for _ in 0..iterations {
        let value = if retired {
            let locale = black_box(locale).clone();
            bundles.get(&locale)
        } else {
            bundles.get(black_box(locale).as_str())
        };
        checksum ^= value.map_or(0, |value| value.len());
    }
    black_box(checksum);
    started.elapsed().as_nanos().max(1)
}

fn percentile(samples: &[u128], percentile: usize) -> u128 {
    let mut sorted = samples.to_vec();
    sorted.sort_unstable();
    sorted[(sorted.len() * percentile).div_ceil(100).saturating_sub(1)]
}

fn csv(samples: &[u128]) -> String {
    samples
        .iter()
        .map(u128::to_string)
        .collect::<Vec<_>>()
        .join(",")
}
