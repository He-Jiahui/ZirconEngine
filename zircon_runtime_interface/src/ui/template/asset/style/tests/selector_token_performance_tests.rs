//! 以 Vec<char> 旧 tokenizer 对照 UTF-8 切片 token 结果，覆盖多字节名称和非法空 token，防止容量优化改变语义。
use std::{hint::black_box, time::Instant};

use super::{parse_compound_tokens, UiAssetError, UiSelectorToken};

fn parse_compound_tokens_allocating_chars(
    input: &str,
) -> Result<Vec<UiSelectorToken>, UiAssetError> {
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
            return Err(UiAssetError::InvalidSelector(input.to_string()));
        }

        match prefix {
            '.' => tokens.push(UiSelectorToken::Class(value)),
            '#' => tokens.push(UiSelectorToken::Id(value)),
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
            ':' => tokens.push(UiSelectorToken::State(value)),
            _ => tokens.push(UiSelectorToken::Type(value)),
        }

        index = end;
    }

    Ok(tokens)
}

#[test]
fn sliced_selector_tokens_preserve_allocating_char_results() {
    for input in [
        "Button.primary#confirm:hover:part(label)",
        "\u{63a7}\u{4ef6}.\u{4e3b}\u{8981}#\u{786e}\u{8ba4}:\u{805a}\u{7126}",
        ":host",
        "Type",
        ".",
        ":part()",
        "Type..class",
    ] {
        assert_eq!(
            parse_compound_tokens(input),
            parse_compound_tokens_allocating_chars(input),
            "selector compound {input:?}",
        );
    }
}

#[test]
#[ignore = "release-only sliced selector tokenization benchmark"]
fn runtime_interface03_batch41_sliced_selector_token_release_benchmark() {
    const ITERATIONS: usize = 100_000;
    const SAMPLE_COUNT: usize = 11;
    let mut input = "LongWidgetType".repeat(16);
    for index in 0..32 {
        input.push_str(&format!(".class{index:02}{}", "x".repeat(32)));
    }
    input.push_str("#primary:hover:part(content)");
    let mut allocating_samples = Vec::with_capacity(SAMPLE_COUNT);
    let mut sliced_samples = Vec::with_capacity(SAMPLE_COUNT);

    for sample in 0..SAMPLE_COUNT {
        let measure_allocating = || {
            let started = Instant::now();
            for _ in 0..ITERATIONS {
                black_box(parse_compound_tokens_allocating_chars(black_box(&input)).unwrap());
            }
            started.elapsed().as_nanos()
        };
        let measure_sliced = || {
            let started = Instant::now();
            for _ in 0..ITERATIONS {
                black_box(parse_compound_tokens(black_box(&input)).unwrap());
            }
            started.elapsed().as_nanos()
        };
        if sample % 2 == 0 {
            allocating_samples.push(measure_allocating());
            sliced_samples.push(measure_sliced());
        } else {
            sliced_samples.push(measure_sliced());
            allocating_samples.push(measure_allocating());
        }
    }

    allocating_samples.sort_unstable();
    sliced_samples.sort_unstable();
    let p95 = SAMPLE_COUNT - 1;
    eprintln!(
        "RUNTIME_INTERFACE03_SLICED_SELECTOR_TOKEN_BENCH_V1 bytes={} iterations={ITERATIONS} samples={SAMPLE_COUNT} allocating_p95_ns={} sliced_p95_ns={}",
        input.len(), allocating_samples[p95], sliced_samples[p95],
    );
    assert!(
        sliced_samples[p95].saturating_mul(5) <= allocating_samples[p95].saturating_mul(4),
        "sliced selector tokenization must improve P95 by at least 20%: allocating={}ns sliced={}ns",
        allocating_samples[p95],
        sliced_samples[p95],
    );
}
