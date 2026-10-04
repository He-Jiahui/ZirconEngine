//! 逐个比较移动读取与克隆读取所得 token 和游标；长标识符基准用于暴露 `Parser::next` 避免字符串克隆的收益。
use super::*;

#[test]
fn parser_token_take_preserves_cloned_sequence() {
    let tokens = vec![
        Token::Ident("control".to_string()),
        Token::String("status text".to_string()),
        Token::Integer(i64::MIN),
        Token::Float(2.5),
        Token::Bool(true),
        Token::Unsupported("?".to_string()),
    ];
    let mut cloned = Parser {
        tokens: tokens.clone(),
        index: 0,
    };
    let mut moved = Parser { tokens, index: 0 };

    while cloned.index < cloned.tokens.len() {
        assert_eq!(moved.next(), cloned.next_cloned());
        assert_eq!(moved.index, cloned.index);
    }
    assert_eq!(moved.next(), cloned.next_cloned());
}

#[test]
#[ignore = "release-only parser token take benchmark"]
fn runtime_interface03_batch30_parser_token_take_release_benchmark() {
    use std::{hint::black_box, time::Instant};

    const LOOKUP_COUNT: usize = 200_000;
    const SAMPLE_COUNT: usize = 11;
    const TOKEN_BYTES: usize = 512;
    let token_text = "x".repeat(TOKEN_BYTES);
    let mut cloned_parser = Parser {
        tokens: vec![Token::Ident(token_text.clone())],
        index: 0,
    };
    let mut moved_parser = Parser {
        tokens: vec![Token::Ident(token_text)],
        index: 0,
    };
    let mut cloned_samples = Vec::with_capacity(SAMPLE_COUNT);
    let mut moved_samples = Vec::with_capacity(SAMPLE_COUNT);

    for sample in 0..SAMPLE_COUNT {
        let mut measure_cloned = || {
            let started = Instant::now();
            for _ in 0..LOOKUP_COUNT {
                cloned_parser.index = 0;
                black_box(cloned_parser.next_cloned());
            }
            started.elapsed().as_nanos()
        };
        let mut measure_moved = || {
            let started = Instant::now();
            for _ in 0..LOOKUP_COUNT {
                moved_parser.index = 0;
                let token = moved_parser
                    .next()
                    .expect("benchmark parser always contains one token");
                black_box(&token);
                moved_parser.tokens[0] = token;
            }
            started.elapsed().as_nanos()
        };
        if sample % 2 == 0 {
            cloned_samples.push(measure_cloned());
            moved_samples.push(measure_moved());
        } else {
            moved_samples.push(measure_moved());
            cloned_samples.push(measure_cloned());
        }
    }

    cloned_samples.sort_unstable();
    moved_samples.sort_unstable();
    let p95 = SAMPLE_COUNT - 1;
    eprintln!(
        "RUNTIME_INTERFACE03_PARSER_TOKEN_TAKE_BENCH_V1 token_bytes={TOKEN_BYTES} lookups={LOOKUP_COUNT} samples={SAMPLE_COUNT} cloned_p95_ns={} moved_p95_ns={}",
        cloned_samples[p95], moved_samples[p95],
    );
    assert!(
        moved_samples[p95].saturating_mul(2) <= cloned_samples[p95],
        "moving parser tokens must improve P95 by at least 50%: cloned={}ns moved={}ns",
        cloned_samples[p95],
        moved_samples[p95],
    );
}
