//! 以逐字符拼接的旧解析器核对切片解析的成功树和错误集合；长 24 段子选择器检验避免临时 String 的 P95 收益。
use std::{hint::black_box, time::Instant};

use super::{
    parse_compound_tokens, skip_whitespace, UiAssetError, UiSelector, UiSelectorCombinator,
    UiSelectorSegment,
};

fn parse_selector_allocating_compounds(input: &str) -> Result<UiSelector, UiAssetError> {
    let mut chars = input.chars().peekable();
    let mut segments = Vec::new();
    let mut combinator = None;

    loop {
        skip_whitespace(&mut chars);
        if chars.peek().is_none() {
            break;
        }

        let mut compound = String::new();
        while let Some(&character) = chars.peek() {
            if character.is_whitespace() || character == '>' {
                break;
            }
            compound.push(character);
            let _ = chars.next();
        }

        if compound.is_empty() {
            return Err(UiAssetError::InvalidSelector(input.to_string()));
        }

        segments.push(UiSelectorSegment {
            combinator,
            tokens: parse_compound_tokens(&compound)?,
        });

        let saw_space = skip_whitespace(&mut chars);
        combinator = match chars.peek().copied() {
            Some('>') => {
                let _ = chars.next();
                Some(UiSelectorCombinator::Child)
            }
            Some(_) if saw_space => Some(UiSelectorCombinator::Descendant),
            Some(_) => {
                return Err(UiAssetError::InvalidSelector(format!(
                    "{input}: expected whitespace or '>' between selector compounds"
                )));
            }
            None => None,
        };
    }

    if segments.is_empty() || combinator.is_some() {
        return Err(UiAssetError::InvalidSelector(input.to_string()));
    }

    Ok(UiSelector { segments })
}

#[test]
fn sliced_selector_compounds_preserve_allocating_parser_results() {
    for input in [
        "Button.primary > Label:part(text)",
        "  Window Content.item:hover  >  Label ",
        "\u{63a7}\u{4ef6}.\u{4e3b}\u{8981} > \u{6807}\u{7b7e}:\u{805a}\u{7126}",
        "Button>Label",
        "Button >",
        "> Button",
        "Button >> Label",
        "   ",
    ] {
        let sliced = UiSelector::parse(input);
        let allocating = parse_selector_allocating_compounds(input);
        assert_eq!(sliced.is_err(), allocating.is_err(), "selector {input:?}");
        assert_eq!(sliced.as_ref().ok(), allocating.as_ref().ok());
    }
}

#[test]
#[ignore = "release-only sliced selector compound benchmark"]
fn runtime_interface03_batch42_sliced_selector_compound_release_benchmark() {
    const ITERATIONS: usize = 50_000;
    const SAMPLE_COUNT: usize = 11;
    let segment = format!("Widget{}", ".class-name-with-padding".repeat(16));
    let input = (0..24)
        .map(|index| format!("{segment}#node{index:02}:hover"))
        .collect::<Vec<_>>()
        .join(" > ");
    let mut allocating_samples = Vec::with_capacity(SAMPLE_COUNT);
    let mut sliced_samples = Vec::with_capacity(SAMPLE_COUNT);

    for sample in 0..SAMPLE_COUNT {
        let measure_allocating = || {
            let started = Instant::now();
            for _ in 0..ITERATIONS {
                black_box(parse_selector_allocating_compounds(black_box(&input)).unwrap());
            }
            started.elapsed().as_nanos()
        };
        let measure_sliced = || {
            let started = Instant::now();
            for _ in 0..ITERATIONS {
                black_box(UiSelector::parse(black_box(&input)).unwrap());
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
        "RUNTIME_INTERFACE03_SLICED_SELECTOR_COMPOUND_BENCH_V1 bytes={} segments=24 iterations={ITERATIONS} samples={SAMPLE_COUNT} allocating_p95_ns={} sliced_p95_ns={}",
        input.len(), allocating_samples[p95], sliced_samples[p95],
    );
    assert!(
        sliced_samples[p95].saturating_mul(5) <= allocating_samples[p95].saturating_mul(4),
        "sliced selector compound parsing must improve P95 by at least 20%: allocating={}ns sliced={}ns",
        allocating_samples[p95],
        sliced_samples[p95],
    );
}
