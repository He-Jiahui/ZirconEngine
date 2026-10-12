//! 对照分配式旧 tokenizer 检查边界整数、溢出和小数的 token 与消费位置；忽略的 release 基准量化栈上整数解析的 P95。
use super::*;

#[test]
fn stack_integer_tokens_preserve_allocating_parser_results() {
    for source in [
        "0",
        "-0",
        "42",
        "-42",
        "9223372036854775807",
        "-9223372036854775808",
        "9223372036854775808",
        "-9223372036854775809",
        "-",
        "2.5",
        "-2.5",
        "1.",
    ] {
        let chars = source.chars().collect::<Vec<_>>();
        let mut allocating_index = 0;
        let mut stack_index = 0;
        assert_eq!(
            parse_number_or_ident(&chars, &mut stack_index),
            parse_number_or_ident_allocating(&chars, &mut allocating_index),
            "numeric source: {source}",
        );
        assert_eq!(stack_index, allocating_index, "numeric source: {source}");
    }
}

#[test]
#[ignore = "release-only stack integer token benchmark"]
fn runtime_interface03_batch29_stack_integer_token_release_benchmark() {
    use std::{hint::black_box, time::Instant};

    const LOOKUP_COUNT: usize = 200_000;
    const SAMPLE_COUNT: usize = 11;
    let chars = "9223372036854775807".chars().collect::<Vec<_>>();
    let mut allocating_samples = Vec::with_capacity(SAMPLE_COUNT);
    let mut stack_samples = Vec::with_capacity(SAMPLE_COUNT);

    for sample in 0..SAMPLE_COUNT {
        let measure_allocating = || {
            let started = Instant::now();
            for _ in 0..LOOKUP_COUNT {
                let mut index = 0;
                black_box(parse_number_or_ident_allocating(
                    black_box(&chars),
                    black_box(&mut index),
                ));
            }
            started.elapsed().as_nanos()
        };
        let measure_stack = || {
            let started = Instant::now();
            for _ in 0..LOOKUP_COUNT {
                let mut index = 0;
                black_box(parse_number_or_ident(
                    black_box(&chars),
                    black_box(&mut index),
                ));
            }
            started.elapsed().as_nanos()
        };
        if sample % 2 == 0 {
            allocating_samples.push(measure_allocating());
            stack_samples.push(measure_stack());
        } else {
            stack_samples.push(measure_stack());
            allocating_samples.push(measure_allocating());
        }
    }

    allocating_samples.sort_unstable();
    stack_samples.sort_unstable();
    let p95 = SAMPLE_COUNT - 1;
    eprintln!(
        "RUNTIME_INTERFACE03_STACK_INTEGER_TOKEN_BENCH_V1 lookups={LOOKUP_COUNT} samples={SAMPLE_COUNT} allocating_p95_ns={} stack_p95_ns={}",
        allocating_samples[p95], stack_samples[p95],
    );
    assert!(
        stack_samples[p95].saturating_mul(5) <= allocating_samples[p95].saturating_mul(4),
        "stack integer token must improve P95 by at least 20%: allocating={}ns stack={}ns",
        allocating_samples[p95],
        stack_samples[p95],
    );
}
