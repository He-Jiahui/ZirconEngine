//! 对照无预留容量的 token 构造器，先核对 Unicode、空片段和错误结果；长 compound 基准隔离 Vec 容量预估的收益。
use std::{hint::black_box, time::Instant};

use super::{parse_compound_tokens, UiAssetError, UiSelectorToken};

fn parse_compound_tokens_unpresized(input: &str) -> Result<Vec<UiSelectorToken>, UiAssetError> {
    let mut index = 0;
    let mut tokens = Vec::new();

    while index < input.len() {
        let prefix = input[index..]
            .chars()
            .next()
            .expect("selector index remains on a character boundary");
        let start = if matches!(prefix, '.' | '#' | ':') {
            index + prefix.len_utf8()
        } else {
            index
        };
        let end = input[start..]
            .char_indices()
            .find_map(|(offset, character)| {
                matches!(character, '.' | '#' | ':').then_some(start + offset)
            })
            .unwrap_or(input.len());
        let value = &input[start..end];
        if value.is_empty() {
            return Err(UiAssetError::InvalidSelector(input.to_string()));
        }

        match prefix {
            '.' => tokens.push(UiSelectorToken::Class(value.to_string())),
            '#' => tokens.push(UiSelectorToken::Id(value.to_string())),
            ':' if value == "host" => tokens.push(UiSelectorToken::Host),
            ':' if value.starts_with("part(") && value.ends_with(')') => {
                let part = value
                    .strip_prefix("part(")
                    .and_then(|value| value.strip_suffix(')'))
                    .unwrap_or_default();
                if part.is_empty() {
                    return Err(UiAssetError::InvalidSelector(input.to_string()));
                }
                tokens.push(UiSelectorToken::Part(part.to_string()));
            }
            ':' => tokens.push(UiSelectorToken::State(value.to_string())),
            _ => tokens.push(UiSelectorToken::Type(value.to_string())),
        }
        index = end;
    }

    Ok(tokens)
}

#[test]
fn presized_selector_tokens_preserve_unpresized_results() {
    for input in [
        "Button.primary#confirm:hover:part(label)",
        ".primary.secondary:hover",
        "\u{63a7}\u{4ef6}.\u{4e3b}\u{8981}#\u{786e}\u{8ba4}:\u{805a}\u{7126}",
        ":host",
        "Type",
        "Type..class",
        "",
    ] {
        assert_eq!(
            parse_compound_tokens(input),
            parse_compound_tokens_unpresized(input),
            "selector compound {input:?}",
        );
    }
}

#[test]
#[ignore = "release-only presized selector token benchmark"]
fn runtime_interface03_batch43_presized_selector_token_release_benchmark() {
    const ITERATIONS: usize = 100_000;
    const SAMPLE_COUNT: usize = 11;
    let mut input = String::from("Widget");
    for index in 0..128 {
        input.push_str(&format!(".c{index:03}"));
    }
    input.push_str("#primary:hover:part(content)");
    let mut unpresized_samples = Vec::with_capacity(SAMPLE_COUNT);
    let mut presized_samples = Vec::with_capacity(SAMPLE_COUNT);

    for sample in 0..SAMPLE_COUNT {
        let measure_unpresized = || {
            let started = Instant::now();
            for _ in 0..ITERATIONS {
                black_box(parse_compound_tokens_unpresized(black_box(&input)).unwrap());
            }
            started.elapsed().as_nanos()
        };
        let measure_presized = || {
            let started = Instant::now();
            for _ in 0..ITERATIONS {
                black_box(parse_compound_tokens(black_box(&input)).unwrap());
            }
            started.elapsed().as_nanos()
        };
        if sample % 2 == 0 {
            unpresized_samples.push(measure_unpresized());
            presized_samples.push(measure_presized());
        } else {
            presized_samples.push(measure_presized());
            unpresized_samples.push(measure_unpresized());
        }
    }

    unpresized_samples.sort_unstable();
    presized_samples.sort_unstable();
    let p95 = SAMPLE_COUNT - 1;
    eprintln!(
        "RUNTIME_INTERFACE03_PRESIZED_SELECTOR_TOKEN_BENCH_V1 tokens=132 iterations={ITERATIONS} samples={SAMPLE_COUNT} unpresized_p95_ns={} presized_p95_ns={}",
        unpresized_samples[p95], presized_samples[p95],
    );
    assert!(
        presized_samples[p95].saturating_mul(5) <= unpresized_samples[p95].saturating_mul(4),
        "presized selector tokens must improve P95 by at least 20%: unpresized={}ns presized={}ns",
        unpresized_samples[p95],
        presized_samples[p95],
    );
}
