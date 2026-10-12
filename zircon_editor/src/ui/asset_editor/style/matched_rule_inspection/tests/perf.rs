use std::hint::black_box;
use std::time::Instant;

use super::{parse_selector_tokens, InspectorSelectorToken};

const SAMPLE_PAIRS: usize = 17;
const PARSES_PER_SAMPLE: usize = 2_048;

#[test]
fn optimization_batch_jl_editor651_streaming_selector_tokens_preserve_unicode_and_kinds() {
    let tokens =
        parse_selector_tokens("Button.primary#submit:hover:host").expect("selector tokens");
    assert_eq!(token_signature(&tokens[0]), (0, "Button"));
    assert_eq!(token_signature(&tokens[1]), (1, "primary"));
    assert_eq!(token_signature(&tokens[2]), (2, "submit"));
    assert_eq!(token_signature(&tokens[3]), (3, "hover"));
    assert_eq!(token_signature(&tokens[4]), (4, ""));

    let unicode = parse_selector_tokens("\u{6309}\u{94ae}#\u{63d0}\u{4ea4}")
        .expect("Unicode selector tokens");
    assert_eq!(token_signature(&unicode[0]), (0, "\u{6309}\u{94ae}"));
    assert_eq!(token_signature(&unicode[1]), (2, "\u{63d0}\u{4ea4}"));
}

#[test]
fn optimization_batch_jl_editor651_selector_tokenization_avoids_char_buffer() {
    let source = include_str!("../../matched_rule_inspection.rs");
    let function = source
        .split("fn parse_selector_tokens(")
        .nth(1)
        .expect("selector token parser");

    assert!(function.contains("char_indices()"));
    assert!(!function.contains("let chars: Vec<char> = input.chars().collect();"));
}

#[test]
#[ignore = "managed Windows release performance evidence"]
fn optimization_batch_jl_editor651_streaming_selector_tokenization_bench() {
    let selector = format!(
        "{}#{}.{}:hover:host",
        "Widget".repeat(256),
        "control".repeat(128),
        "class".repeat(128),
    );
    for _ in 0..4 {
        black_box(measure(&selector, false));
        black_box(measure(&selector, true));
    }

    let mut buffered_samples = Vec::with_capacity(SAMPLE_PAIRS);
    let mut streaming_samples = Vec::with_capacity(SAMPLE_PAIRS);
    for pair in 0..SAMPLE_PAIRS {
        if pair % 2 == 0 {
            buffered_samples.push(measure(&selector, false));
            streaming_samples.push(measure(&selector, true));
        } else {
            streaming_samples.push(measure(&selector, true));
            buffered_samples.push(measure(&selector, false));
        }
    }

    let buffered_p50_ns = percentile(&buffered_samples, 50);
    let streaming_p50_ns = percentile(&streaming_samples, 50);
    let buffered_p95_ns = percentile(&buffered_samples, 95);
    let streaming_p95_ns = percentile(&streaming_samples, 95);
    println!(
        "EDITOR651_STREAMING_SELECTOR_TOKENIZATION_BENCH_V1 sample_pairs={SAMPLE_PAIRS} \
parses_per_sample={PARSES_PER_SAMPLE} selector_bytes={} buffered_p50_ns={buffered_p50_ns} \
streaming_p50_ns={streaming_p50_ns} buffered_p95_ns={buffered_p95_ns} \
streaming_p95_ns={streaming_p95_ns} buffered_raw_ns={} streaming_raw_ns={}",
        selector.len(),
        sample_csv(&buffered_samples),
        sample_csv(&streaming_samples),
    );

    assert!(streaming_p95_ns <= buffered_p95_ns * 70 / 100);
}

fn measure(input: &str, streaming: bool) -> u128 {
    let started = Instant::now();
    let mut checksum = 0usize;
    for _ in 0..PARSES_PER_SAMPLE {
        let tokens = if streaming {
            parse_selector_tokens(black_box(input))
        } else {
            legacy_parse_selector_tokens(black_box(input))
        }
        .expect("benchmark selector tokens");
        checksum = checksum.wrapping_add(
            tokens
                .iter()
                .map(|token| token_signature(token).1.len())
                .sum::<usize>(),
        );
        black_box(tokens);
    }
    black_box(checksum);
    started.elapsed().as_nanos().max(1)
}

fn legacy_parse_selector_tokens(input: &str) -> Option<Vec<InspectorSelectorToken>> {
    let chars: Vec<char> = input.chars().collect();
    let mut index = 0;
    let mut tokens = Vec::new();
    while index < chars.len() {
        let prefix = chars[index];
        let start = if matches!(prefix, '.' | '#' | ':') {
            index + 1
        } else {
            index
        };
        let mut end = start;
        while end < chars.len() && !matches!(chars[end], '.' | '#' | ':') {
            end += 1;
        }
        let value: String = chars[start..end].iter().collect();
        if value.is_empty() {
            return None;
        }
        let token = match prefix {
            '.' => InspectorSelectorToken::Class(value),
            '#' => InspectorSelectorToken::Id(value),
            ':' if value == "host" => InspectorSelectorToken::Host,
            ':' => InspectorSelectorToken::State(value),
            _ => InspectorSelectorToken::Type(value),
        };
        tokens.push(token);
        index = end;
    }
    (!tokens.is_empty()).then_some(tokens)
}

fn token_signature(token: &InspectorSelectorToken) -> (u8, &str) {
    match token {
        InspectorSelectorToken::Type(value) => (0, value),
        InspectorSelectorToken::Class(value) => (1, value),
        InspectorSelectorToken::Id(value) => (2, value),
        InspectorSelectorToken::State(value) => (3, value),
        InspectorSelectorToken::Host => (4, ""),
    }
}

fn percentile(samples: &[u128], percentile: usize) -> u128 {
    let mut sorted = samples.to_vec();
    sorted.sort_unstable();
    let rank = (sorted.len() * percentile).div_ceil(100);
    sorted[rank.saturating_sub(1)]
}

fn sample_csv(samples: &[u128]) -> String {
    samples
        .iter()
        .map(u128::to_string)
        .collect::<Vec<_>>()
        .join(",")
}
