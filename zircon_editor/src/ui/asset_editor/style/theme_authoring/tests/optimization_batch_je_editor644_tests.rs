use std::collections::BTreeMap;
use std::hint::black_box;
use std::time::Instant;

const TOKEN_COUNT: usize = 65_536;
const SAMPLE_PAIRS: usize = 17;
const THEME_REFERENCE: &str = "batch-644-theme";

#[test]
fn optimization_batch_je_editor644_reuses_bulk_theme_token_adoption() {
    let source = include_str!("../../theme_authoring.rs");
    let comparison = source
        .split("pub(crate) fn adopt_imported_theme_compare_diffs")
        .nth(1)
        .expect("compare-diff adoption remains present")
        .split("pub(crate) fn prune_imported_theme_compare_duplicates")
        .next()
        .expect("compare-diff adoption remains bounded");

    assert!(comparison.contains(
        "let mut adopted = adopt_imported_theme_tokens(document, imported_styles, reference);"
    ));
    assert!(!comparison.contains("for (token_name, imported_value)"));
    assert!(!comparison.contains("adopt_imported_theme_token("));
}

#[test]
#[ignore = "release performance gate"]
fn optimization_batch_je_editor644_direct_bulk_theme_token_adoption_benchmark() {
    let imported_tokens = (0..TOKEN_COUNT)
        .map(|index| (index, index.wrapping_mul(31)))
        .collect::<BTreeMap<_, _>>();
    let imported_styles = BTreeMap::from([(THEME_REFERENCE.to_string(), imported_tokens)]);
    let local_tokens = (0..TOKEN_COUNT / 2)
        .map(|index| (index, index.wrapping_mul(31)))
        .collect::<BTreeMap<_, _>>();
    assert_eq!(
        legacy_adopt(&local_tokens, &imported_styles),
        direct_adopt(&local_tokens, &imported_styles)
    );

    for _ in 0..4 {
        black_box(measure_adoption(&local_tokens, &imported_styles, false));
        black_box(measure_adoption(&local_tokens, &imported_styles, true));
    }

    let mut repeated_helper_samples = Vec::with_capacity(SAMPLE_PAIRS);
    let mut direct_samples = Vec::with_capacity(SAMPLE_PAIRS);
    for pair in 0..SAMPLE_PAIRS {
        if pair % 2 == 0 {
            repeated_helper_samples.push(measure_adoption(&local_tokens, &imported_styles, false));
            direct_samples.push(measure_adoption(&local_tokens, &imported_styles, true));
        } else {
            direct_samples.push(measure_adoption(&local_tokens, &imported_styles, true));
            repeated_helper_samples.push(measure_adoption(&local_tokens, &imported_styles, false));
        }
    }

    let repeated_helper_p95 = percentile(&repeated_helper_samples, 95);
    let direct_p95 = percentile(&direct_samples, 95);
    let improvement_percent = repeated_helper_p95
        .saturating_sub(direct_p95)
        .saturating_mul(100)
        / repeated_helper_p95.max(1);
    println!(
        "EDITOR644_DIRECT_BULK_THEME_TOKEN_ADOPTION_BENCH_V1 sample_pairs={SAMPLE_PAIRS} token_count={TOKEN_COUNT} repeated_helper_ns={} direct_ns={} repeated_helper_p95_ns={repeated_helper_p95} direct_p95_ns={direct_p95} improvement_percent={improvement_percent} threshold_percent=20",
        csv(&repeated_helper_samples),
        csv(&direct_samples),
    );
    assert!(direct_p95 <= repeated_helper_p95 * 80 / 100);
}

fn measure_adoption(
    local_tokens: &BTreeMap<usize, usize>,
    imported_styles: &BTreeMap<String, BTreeMap<usize, usize>>,
    direct: bool,
) -> u128 {
    let started = Instant::now();
    let adopted = if direct {
        direct_adopt(local_tokens, imported_styles)
    } else {
        legacy_adopt(local_tokens, imported_styles)
    };
    black_box(adopted);
    started.elapsed().as_nanos().max(1)
}

fn legacy_adopt(
    local_tokens: &BTreeMap<usize, usize>,
    imported_styles: &BTreeMap<String, BTreeMap<usize, usize>>,
) -> (usize, BTreeMap<usize, usize>) {
    let mut local_tokens = local_tokens.clone();
    let imported_tokens = imported_styles
        .get(THEME_REFERENCE)
        .expect("synthetic imported theme");
    let mut adopted = 0usize;
    for (token_name, imported_value) in imported_tokens {
        if local_tokens.get(token_name) == Some(imported_value) {
            continue;
        }
        adopted += usize::from(legacy_adopt_token(
            &mut local_tokens,
            imported_styles,
            token_name,
        ));
    }
    (adopted, local_tokens)
}

fn legacy_adopt_token(
    local_tokens: &mut BTreeMap<usize, usize>,
    imported_styles: &BTreeMap<String, BTreeMap<usize, usize>>,
    token_name: &usize,
) -> bool {
    let imported_tokens = imported_styles
        .get(THEME_REFERENCE)
        .expect("synthetic imported theme");
    let imported_value = imported_tokens
        .get(token_name)
        .expect("synthetic imported token");
    if local_tokens.get(token_name) == Some(imported_value) {
        return false;
    }
    local_tokens.insert(*token_name, *imported_value);
    true
}

fn direct_adopt(
    local_tokens: &BTreeMap<usize, usize>,
    imported_styles: &BTreeMap<String, BTreeMap<usize, usize>>,
) -> (usize, BTreeMap<usize, usize>) {
    let mut local_tokens = local_tokens.clone();
    let imported_tokens = imported_styles
        .get(THEME_REFERENCE)
        .expect("synthetic imported theme");
    let mut adopted = 0usize;
    for (token_name, imported_value) in imported_tokens {
        if local_tokens.get(token_name) == Some(imported_value) {
            continue;
        }
        local_tokens.insert(*token_name, *imported_value);
        adopted += 1;
    }
    (adopted, local_tokens)
}

fn percentile(samples: &[u128], percentile: usize) -> u128 {
    let mut sorted = samples.to_vec();
    sorted.sort_unstable();
    let rank = (sorted.len() * percentile).div_ceil(100);
    sorted[rank.saturating_sub(1)]
}

fn csv(samples: &[u128]) -> String {
    samples
        .iter()
        .map(u128::to_string)
        .collect::<Vec<_>>()
        .join(",")
}
