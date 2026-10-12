use super::*;

fn parse_string_growing(input: &str) -> Result<String, UiBindingParseError> {
    let mut parser = BindingParser::new(input);
    parser.expect('"')?;
    let mut output = String::new();
    while let Some(ch) = parser.peek_char() {
        parser.index += ch.len_utf8();
        match ch {
            '"' => return Ok(output),
            '\\' => {
                let escaped = parser
                    .peek_char()
                    .ok_or(UiBindingParseError::InvalidEscape)?;
                parser.index += escaped.len_utf8();
                output.push(match escaped {
                    '"' => '"',
                    '\\' => '\\',
                    'n' => '\n',
                    'r' => '\r',
                    't' => '\t',
                    _ => return Err(UiBindingParseError::InvalidEscape),
                });
            }
            other => output.push(other),
        }
    }
    Err(UiBindingParseError::UnterminatedString)
}

fn parse_string_sliced(input: &str) -> Result<String, UiBindingParseError> {
    BindingParser::new(input).parse_string()
}

#[test]
fn runtime_interface03_batch49_50_sliced_binding_strings_preserve_growing_results() {
    for input in [
        "\"\"",
        "\"plain binding value\"",
        "\"unicode 界面 value\"",
        "\"line\\nquote\\\"slash\\\\tab\\treturn\\r\"",
        "\"trailing\"remaining input",
        "\"unterminated",
        "\"invalid\\x\"",
        "not quoted",
    ] {
        assert_eq!(parse_string_sliced(input), parse_string_growing(input));
    }
}

fn p95(mut samples: Vec<u128>) -> u128 {
    samples.sort_unstable();
    samples[(samples.len() * 95).div_ceil(100) - 1]
}

#[test]
#[ignore = "release-only sliced unescaped binding-string benchmark"]
fn runtime_interface03_batch49_50_sliced_unescaped_binding_string_release_benchmark() {
    use std::{hint::black_box, time::Instant};

    const PARSE_COUNT: usize = 50_000;
    const SAMPLE_COUNT: usize = 11;
    let input = format!("\"{}\"", "runtime_interface_binding_value_".repeat(32));
    let mut growing_samples = Vec::with_capacity(SAMPLE_COUNT);
    let mut sliced_samples = Vec::with_capacity(SAMPLE_COUNT);

    for sample in 0..SAMPLE_COUNT {
        let measure_growing = || {
            let started = Instant::now();
            for _ in 0..PARSE_COUNT {
                black_box(parse_string_growing(black_box(&input)).unwrap());
            }
            started.elapsed().as_nanos()
        };
        let measure_sliced = || {
            let started = Instant::now();
            for _ in 0..PARSE_COUNT {
                black_box(parse_string_sliced(black_box(&input)).unwrap());
            }
            started.elapsed().as_nanos()
        };
        if sample % 2 == 0 {
            growing_samples.push(measure_growing());
            sliced_samples.push(measure_sliced());
        } else {
            sliced_samples.push(measure_sliced());
            growing_samples.push(measure_growing());
        }
    }

    let growing_p95_ns = p95(growing_samples);
    let sliced_p95_ns = p95(sliced_samples);
    eprintln!(
        "RUNTIME_INTERFACE03_SLICED_BINDING_STRING_BENCH_V1 parses={PARSE_COUNT} bytes={} samples={SAMPLE_COUNT} growing_p95_ns={growing_p95_ns} sliced_p95_ns={sliced_p95_ns}",
        input.len(),
    );
    assert!(
        sliced_p95_ns.saturating_mul(5) <= growing_p95_ns.saturating_mul(4),
        "sliced unescaped binding-string parsing must improve P95 by at least 20%: growing={growing_p95_ns}ns sliced={sliced_p95_ns}ns",
    );
}

#[test]
#[ignore = "release-only chunked escaped binding-string benchmark"]
fn runtime_interface03_batch49_50_chunked_escaped_binding_string_release_benchmark() {
    use std::{hint::black_box, time::Instant};

    const PARSE_COUNT: usize = 50_000;
    const SAMPLE_COUNT: usize = 11;
    let chunk = "runtime_interface_binding_value_".repeat(8);
    let input = format!("\"{chunk}\\n{chunk}\\\"{chunk}\\\\{chunk}\"");
    let mut growing_samples = Vec::with_capacity(SAMPLE_COUNT);
    let mut chunked_samples = Vec::with_capacity(SAMPLE_COUNT);

    for sample in 0..SAMPLE_COUNT {
        let measure_growing = || {
            let started = Instant::now();
            for _ in 0..PARSE_COUNT {
                black_box(parse_string_growing(black_box(&input)).unwrap());
            }
            started.elapsed().as_nanos()
        };
        let measure_chunked = || {
            let started = Instant::now();
            for _ in 0..PARSE_COUNT {
                black_box(parse_string_sliced(black_box(&input)).unwrap());
            }
            started.elapsed().as_nanos()
        };
        if sample % 2 == 0 {
            growing_samples.push(measure_growing());
            chunked_samples.push(measure_chunked());
        } else {
            chunked_samples.push(measure_chunked());
            growing_samples.push(measure_growing());
        }
    }

    let growing_p95_ns = p95(growing_samples);
    let chunked_p95_ns = p95(chunked_samples);
    eprintln!(
        "RUNTIME_INTERFACE03_CHUNKED_BINDING_STRING_BENCH_V1 parses={PARSE_COUNT} bytes={} escapes=3 samples={SAMPLE_COUNT} growing_p95_ns={growing_p95_ns} chunked_p95_ns={chunked_p95_ns}",
        input.len(),
    );
    assert!(
        chunked_p95_ns.saturating_mul(5) <= growing_p95_ns.saturating_mul(4),
        "chunked escaped binding-string parsing must improve P95 by at least 20%: growing={growing_p95_ns}ns chunked={chunked_p95_ns}ns",
    );
}
