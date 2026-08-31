use std::hint::black_box;
use std::time::Instant;

use super::is_forbidden_line_start;

const CANDIDATE_COUNT: usize = 4_096;
const SAMPLE_COUNT: usize = 17;
const LEGACY_FORBIDDEN_LINE_START: &[char] = &[
    '、', '。', '，', '．', '・', '：', '；', '！', '？', '）', '］', '｝', '｠', '】', '〕', '〉',
    '》', '」', '』', '〗', '〙', '〛', '’', '”', '〟', '〞', 'ぁ', 'ぃ', 'ぅ', 'ぇ', 'ぉ', 'っ',
    'ゃ', 'ゅ', 'ょ', 'ゎ', 'ゕ', 'ゖ', 'ァ', 'ィ', 'ゥ', 'ェ', 'ォ', 'ッ', 'ャ', 'ュ', 'ョ', 'ヮ',
    'ヵ', 'ヶ', 'ㇰ', 'ㇱ', 'ㇲ', 'ㇳ', 'ㇴ', 'ㇵ', 'ㇶ', 'ㇷ', 'ㇸ', 'ㇹ', 'ㇺ', 'ㇻ', 'ㇼ', 'ㇽ',
    'ㇾ', 'ㇿ', '｡', '｣', '､', '･', 'ｧ', 'ｨ', 'ｩ', 'ｪ', 'ｫ', 'ｯ', 'ｬ', 'ｭ', 'ｮ', 'ｰ', 'ﾞ', 'ﾟ', 'ー',
    '々', '〻', 'ゝ', 'ゞ', 'ヽ', 'ヾ', '゛', '゜', '‐', '〜', '゠', '–',
];
const LEGACY_SCALAR_PROBES: usize = CANDIDATE_COUNT * LEGACY_FORBIDDEN_LINE_START.len();
const OPTIMIZED_MEMBERSHIP_DISPATCHES: usize = CANDIDATE_COUNT;

#[test]
fn optimization_batch_20260826bl_kinsoku_match_dispatch_preserves_membership() {
    assert_eq!(LEGACY_FORBIDDEN_LINE_START.len(), 95);
    for character in LEGACY_FORBIDDEN_LINE_START.iter().copied() {
        assert!(
            is_forbidden_line_start(character),
            "legacy forbidden line-start character {character:?} must remain forbidden"
        );
    }
    for character in ['A', 'あ', 'ア', '中', '한'] {
        assert!(!is_forbidden_line_start(character));
    }
}

#[test]
fn optimization_batch_20260826bl_kinsoku_match_dispatch_eliminates_table_scans() {
    const SOURCE: &str = include_str!("../kinsoku.rs");
    let dispatch = SOURCE
        .split("fn is_forbidden_line_start")
        .nth(1)
        .and_then(|tail| tail.split("fn is_forbidden_line_end").next())
        .expect("forbidden line-start dispatch body");

    assert_eq!(LEGACY_SCALAR_PROBES, 389_120);
    assert_eq!(OPTIMIZED_MEMBERSHIP_DISPATCHES, 4_096);
    assert!(dispatch.contains("matches!("));
    assert!(!dispatch.contains(".contains("));
}

#[test]
#[ignore = "release-only performance evidence"]
fn optimization_batch_20260826bl_kinsoku_match_dispatch_p95() {
    let candidates = (0..CANDIDATE_COUNT)
        .map(|index| char::from_u32(b'A' as u32 + (index % 26) as u32).unwrap())
        .collect::<Vec<_>>();
    let (legacy_samples, optimized_samples) = benchmark_paired_samples::<SAMPLE_COUNT>(
        || legacy_frame(black_box(&candidates)),
        || optimized_frame(black_box(&candidates)),
    );
    assert_eq!(legacy_frame(&candidates), optimized_frame(&candidates));

    let legacy_p50 = percentile(&legacy_samples, 50);
    let legacy_p95 = percentile(&legacy_samples, 95);
    let optimized_p50 = percentile(&optimized_samples, 50);
    let optimized_p95 = percentile(&optimized_samples, 95);
    println!(
        "PERF_RESULT RUNTIME81_KINSOKU_MATCH_DISPATCH_BENCH_V1 candidates={CANDIDATE_COUNT} samples={SAMPLE_COUNT} sample_order=alternating legacy_scalar_probes={LEGACY_SCALAR_PROBES} optimized_membership_dispatches={OPTIMIZED_MEMBERSHIP_DISPATCHES} deterministic_membership_work_reduction_percent=98.9474 legacy_p50_ns={legacy_p50} optimized_p50_ns={optimized_p50} legacy_p95_ns={legacy_p95} optimized_p95_ns={optimized_p95} legacy_samples_ns={} optimized_samples_ns={}",
        join_samples(&legacy_samples),
        join_samples(&optimized_samples),
    );
    assert!(
        optimized_p95 * 2 <= legacy_p95,
        "optimized P95 {optimized_p95}ns must be at least 50% below legacy P95 {legacy_p95}ns"
    );
}

fn legacy_frame(candidates: &[char]) -> usize {
    candidates
        .iter()
        .copied()
        .filter(|character| LEGACY_FORBIDDEN_LINE_START.contains(character))
        .count()
}

fn optimized_frame(candidates: &[char]) -> usize {
    candidates
        .iter()
        .copied()
        .filter(|character| is_forbidden_line_start(*character))
        .count()
}

fn benchmark_paired_samples<const N: usize>(
    mut legacy: impl FnMut() -> usize,
    mut optimized: impl FnMut() -> usize,
) -> (Vec<u128>, Vec<u128>) {
    black_box(legacy());
    black_box(optimized());
    let mut legacy_samples = Vec::with_capacity(N);
    let mut optimized_samples = Vec::with_capacity(N);
    for index in 0..N {
        if index % 2 == 0 {
            legacy_samples.push(measure(&mut legacy));
            optimized_samples.push(measure(&mut optimized));
        } else {
            optimized_samples.push(measure(&mut optimized));
            legacy_samples.push(measure(&mut legacy));
        }
    }
    (legacy_samples, optimized_samples)
}

fn measure(operation: &mut impl FnMut() -> usize) -> u128 {
    let started = Instant::now();
    black_box(operation());
    started.elapsed().as_nanos()
}

fn percentile(samples: &[u128], percentile: usize) -> u128 {
    let mut sorted = samples.to_vec();
    sorted.sort_unstable();
    sorted[(sorted.len() - 1) * percentile / 100]
}

fn join_samples(samples: &[u128]) -> String {
    samples
        .iter()
        .map(u128::to_string)
        .collect::<Vec<_>>()
        .join(",")
}
