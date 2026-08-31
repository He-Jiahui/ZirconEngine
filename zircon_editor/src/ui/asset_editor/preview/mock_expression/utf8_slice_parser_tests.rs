use std::hint::black_box;
use std::time::Instant;

use super::*;

const SAMPLE_PAIRS: usize = 17;

#[test]
fn optimization_batch_20260826ap_preview_expression_parser_preserves_utf8_segments() {
    let parsed = parse_preview_mock_reference(
        " self . inventory [ 12 ] [\"display name\"] ['locale'] .\u{503c} ",
    )
    .expect("valid preview reference");

    assert_eq!(parsed.node_reference, "self");
    assert_eq!(parsed.property, "inventory");
    assert_eq!(
        parsed.nested_segments,
        vec!["12", "display name", "locale", "\u{503c}"]
    );

    let mut path = parsed.property.clone();
    for segment in &parsed.nested_segments {
        append_expression_path_segment(&mut path, segment);
    }
    assert_eq!(path, "inventory[12][\"display name\"].locale[\"\u{503c}\"]");

    assert!(parse_preview_mock_reference("").is_none());
    assert!(parse_preview_mock_reference("self").is_none());
    assert!(parse_preview_mock_reference("self.items[").is_none());
    assert!(parse_preview_mock_reference("self.items[]").is_none());
    assert!(parse_preview_mock_reference("self.items]").is_none());
}

#[test]
fn optimization_batch_20260826ap_preview_expression_parser_avoids_char_projection() {
    let source = include_str!("../mock_expression.rs");
    let parser = bounded_function(
        source,
        "fn parse_preview_mock_reference_segments",
        "fn is_identifier_segment",
    );

    assert!(!parser.contains("collect::<Vec<_>>()"));
    assert!(!parser.contains("chars: &[char]"));
    assert!(!parser.contains("let chars ="));
    assert!(parser.contains("reference.as_bytes()"));
    assert!(parser.contains("len_utf8()"));
    assert!(parser.contains("&reference[start..*index]"));
}

#[test]
#[ignore = "release performance evidence"]
fn optimization_batch_20260826ap_preview_expression_utf8_slice_parser_p95() {
    const IDENTIFIER_CHARS: usize = 4_096;
    const REFERENCES: usize = 512;
    let reference = format!(
        "{}.property[7][\"display name\"]",
        "\u{754c}".repeat(IDENTIFIER_CHARS)
    );

    let mut legacy_ns = Vec::with_capacity(SAMPLE_PAIRS);
    let mut optimized_ns = Vec::with_capacity(SAMPLE_PAIRS);
    for sample_index in 0..SAMPLE_PAIRS {
        if sample_index % 2 == 0 {
            legacy_ns.push(measure_ns(REFERENCES, || {
                legacy_parse_reference_segments(&reference)
            }));
            optimized_ns.push(measure_ns(REFERENCES, || {
                parse_preview_mock_reference(&reference).map(parsed_width)
            }));
        } else {
            optimized_ns.push(measure_ns(REFERENCES, || {
                parse_preview_mock_reference(&reference).map(parsed_width)
            }));
            legacy_ns.push(measure_ns(REFERENCES, || {
                legacy_parse_reference_segments(&reference)
            }));
        }
    }

    let legacy_p50_ns = nearest_rank(&legacy_ns, 50);
    let legacy_p95_ns = nearest_rank(&legacy_ns, 95);
    let optimized_p50_ns = nearest_rank(&optimized_ns, 50);
    let optimized_p95_ns = nearest_rank(&optimized_ns, 95);
    assert!(
        optimized_p95_ns.saturating_mul(5) <= legacy_p95_ns.saturating_mul(4),
        "UTF-8 slice parser P95 must be at least 20% below char-vector projection: legacy={legacy_p95_ns}ns optimized={optimized_p95_ns}ns"
    );

    println!(
        "EDITOR23_PREVIEW_EXPRESSION_UTF8_SLICE_PARSER_BENCH_V1 identifier_chars={IDENTIFIER_CHARS} reference_bytes={} references_per_sample={REFERENCES} sample_pairs={SAMPLE_PAIRS} pair_order=alternating_legacy_even legacy_first_pairs=9 optimized_first_pairs=8 legacy_char_buffer_entries_per_reference={} optimized_char_buffer_entries_per_reference=0 legacy_p50_ns={legacy_p50_ns} legacy_p95_ns={legacy_p95_ns} optimized_p50_ns={optimized_p50_ns} optimized_p95_ns={optimized_p95_ns} legacy_ns={} optimized_ns={}",
        reference.len(),
        reference.chars().count(),
        join_samples(&legacy_ns),
        join_samples(&optimized_ns),
    );
}

fn parsed_width(parsed: ParsedPreviewMockExpression) -> usize {
    parsed.node_reference.len()
        + parsed.property.len()
        + parsed
            .nested_segments
            .iter()
            .map(String::len)
            .sum::<usize>()
}

fn legacy_parse_reference_segments(reference: &str) -> Option<usize> {
    let trimmed = reference.trim();
    if trimmed.is_empty() {
        return None;
    }

    let chars = trimmed.chars().collect::<Vec<_>>();
    let mut index = 0usize;
    let mut segments = Vec::new();
    while index < chars.len() {
        legacy_skip_whitespace(&chars, &mut index);
        if index >= chars.len() {
            break;
        }
        match chars[index] {
            '.' => index += 1,
            '[' => {
                let segment = legacy_bracket_segment(&chars, &mut index)?;
                if segment.is_empty() {
                    return None;
                }
                segments.push(segment);
            }
            ']' => return None,
            _ => {
                let start = index;
                while index < chars.len() && !matches!(chars[index], '.' | '[' | ']') {
                    index += 1;
                }
                let segment = chars[start..index]
                    .iter()
                    .collect::<String>()
                    .trim()
                    .to_string();
                if segment.is_empty() {
                    return None;
                }
                segments.push(segment);
            }
        }
    }
    (segments.len() >= 2).then(|| segments.iter().map(String::len).sum())
}

fn legacy_bracket_segment(chars: &[char], index: &mut usize) -> Option<String> {
    *index += 1;
    legacy_skip_whitespace(chars, index);
    if *index >= chars.len() {
        return None;
    }
    let segment = match chars[*index] {
        quote @ ('"' | '\'') => {
            *index += 1;
            let start = *index;
            while *index < chars.len() && chars[*index] != quote {
                *index += 1;
            }
            if *index >= chars.len() {
                return None;
            }
            let segment = chars[start..*index].iter().collect::<String>();
            *index += 1;
            segment
        }
        _ => {
            let start = *index;
            while *index < chars.len() && chars[*index] != ']' {
                *index += 1;
            }
            if *index >= chars.len() {
                return None;
            }
            chars[start..*index]
                .iter()
                .collect::<String>()
                .trim()
                .to_string()
        }
    };
    legacy_skip_whitespace(chars, index);
    if *index >= chars.len() || chars[*index] != ']' {
        return None;
    }
    *index += 1;
    Some(segment)
}

fn legacy_skip_whitespace(chars: &[char], index: &mut usize) {
    while *index < chars.len() && chars[*index].is_whitespace() {
        *index += 1;
    }
}

fn measure_ns(iterations: usize, mut operation: impl FnMut() -> Option<usize>) -> u128 {
    let started = Instant::now();
    let mut checksum = 0usize;
    for _ in 0..iterations {
        checksum = checksum.wrapping_add(black_box(operation()).expect("valid reference"));
    }
    black_box(checksum);
    started.elapsed().as_nanos()
}

fn bounded_function<'a>(source: &'a str, start: &str, end: &str) -> &'a str {
    source
        .split(start)
        .nth(1)
        .expect("function start")
        .split(end)
        .next()
        .expect("function end")
}

fn nearest_rank(samples: &[u128], percentile: usize) -> u128 {
    let mut sorted = samples.to_vec();
    sorted.sort_unstable();
    let rank = sorted.len().saturating_mul(percentile).div_ceil(100);
    sorted[rank.saturating_sub(1)]
}

fn join_samples(samples: &[u128]) -> String {
    samples
        .iter()
        .map(u128::to_string)
        .collect::<Vec<_>>()
        .join(",")
}
