use std::hint::black_box;
use std::time::{Duration, Instant};

use zircon_runtime_interface::ui::component::UiValue;

use super::{
    flags_source, float_source, literal_source, string_source, typed_string_source, vector_source,
};

const PERFORMANCE_MARKER: &str = "RUNTIME871_BINDING_LITERAL_SINGLE_BUFFER_BENCH_V1";
const SAMPLE_PAIRS: usize = 101;
const LITERALS_PER_SAMPLE: usize = 128;

#[test]
fn runtime871_binding_literal_single_buffer_preserves_exact_text() {
    let values = vec![
        "plain".to_string(),
        "quote\"slash\\".to_string(),
        "line\n\t".to_string(),
        "\u{6e32}\u{67d3}".to_string(),
    ];
    let flags = UiValue::Flags(values.clone());
    let expected = r#"flags("plain", "quote\"slash\\", "line\n\t", "\u{6e32}\u{67d3}")"#;
    // BUG: [CR-R02-runtime_wave5_template_compile_assets-0002] 输入包含真实中文，raw 期望却保留字面 Unicode 转义；append_string_source 原样追加非控制字符，本断言因此失败。
    assert_eq!(literal_source(&flags).as_deref(), Some(expected));
    assert_eq!(flags_source(&values), legacy_flags_source(&values));

    for value in [
        "",
        "plain",
        "quote\"slash\\",
        "line\n\r\t",
        "control\u{0001}\u{001f}",
        "\u{6e32}\u{67d3}",
    ] {
        assert_eq!(string_source(value), legacy_string_source(value));
        assert_eq!(
            typed_string_source("enum", value),
            format!("enum({})", legacy_string_source(value))
        );
    }

    let vector = [1.0, -2.5, 0.0, 42.25];
    assert_eq!(
        vector_source("vec4", &vector),
        legacy_vector_source("vec4", &vector)
    );
    assert_eq!(
        vector_source("vec4", &vector).as_deref(),
        Some("vec4(1.0, -2.5, 0.0, 42.25)")
    );
    assert_eq!(float_source(12.0).as_deref(), Some("12.0"));
    assert_eq!(float_source(f64::INFINITY), None);
    assert_eq!(vector_source("vec2", &[1.0, f64::NAN]), None);
}

#[test]
#[ignore = "release-only binding literal performance gate"]
fn runtime871_binding_literal_single_buffer_release_performance() {
    let values = (0..64)
        .map(|index| format!("flag_\"escaped\\value_{index:04}\n"))
        .collect::<Vec<_>>();
    assert_eq!(flags_source(&values), legacy_flags_source(&values));

    for _ in 0..8 {
        black_box(render_batch(&values, legacy_flags_source));
        black_box(render_batch(&values, flags_source));
    }

    let mut legacy_samples = Vec::with_capacity(SAMPLE_PAIRS);
    let mut optimized_samples = Vec::with_capacity(SAMPLE_PAIRS);
    for sample in 0..SAMPLE_PAIRS {
        if sample % 2 == 0 {
            legacy_samples.push(measure(|| render_batch(&values, legacy_flags_source)));
            optimized_samples.push(measure(|| render_batch(&values, flags_source)));
        } else {
            optimized_samples.push(measure(|| render_batch(&values, flags_source)));
            legacy_samples.push(measure(|| render_batch(&values, legacy_flags_source)));
        }
    }

    let legacy_p50_ns = percentile_ns(&mut legacy_samples, 50);
    let legacy_p95_ns = percentile_ns(&mut legacy_samples, 95);
    let legacy_p99_ns = percentile_ns(&mut legacy_samples, 99);
    let optimized_p50_ns = percentile_ns(&mut optimized_samples, 50);
    let optimized_p95_ns = percentile_ns(&mut optimized_samples, 95);
    let optimized_p99_ns = percentile_ns(&mut optimized_samples, 99);
    println!(
        "{PERFORMANCE_MARKER} legacy_p50_ns={legacy_p50_ns} optimized_p50_ns={optimized_p50_ns} legacy_p95_ns={legacy_p95_ns} optimized_p95_ns={optimized_p95_ns} legacy_p99_ns={legacy_p99_ns} optimized_p99_ns={optimized_p99_ns} sample_pairs={SAMPLE_PAIRS} literals_per_sample={LITERALS_PER_SAMPLE} values_per_literal=64 legacy_child_strings_per_sample=8192 optimized_child_strings_per_sample=0 legacy_vector_slots_per_sample=8192 optimized_vector_slots_per_sample=0"
    );
    assert!(
        optimized_p95_ns.saturating_mul(100) <= legacy_p95_ns.saturating_mul(110),
        "single-buffer P95 {optimized_p95_ns}ns must be at most 110% of collect/join P95 {legacy_p95_ns}ns"
    );
}

fn legacy_flags_source(values: &[String]) -> String {
    format!(
        "flags({})",
        values
            .iter()
            .map(|value| legacy_string_source(value))
            .collect::<Vec<_>>()
            .join(", ")
    )
}

fn legacy_vector_source<const N: usize>(constructor: &str, values: &[f64; N]) -> Option<String> {
    let values = values
        .iter()
        .map(|value| legacy_float_source(*value))
        .collect::<Option<Vec<_>>>()?;
    Some(format!("{constructor}({})", values.join(", ")))
}

fn legacy_string_source(value: &str) -> String {
    let mut source = String::with_capacity(value.len() + 2);
    source.push('"');
    for ch in value.chars() {
        match ch {
            '"' => source.push_str("\\\""),
            '\\' => source.push_str("\\\\"),
            '\n' => source.push_str("\\n"),
            '\r' => source.push_str("\\r"),
            '\t' => source.push_str("\\t"),
            '\u{0008}' => source.push_str("\\b"),
            '\u{000c}' => source.push_str("\\f"),
            ch if ch.is_control() => {
                use std::fmt::Write as _;
                write!(&mut source, "\\u{:04x}", ch as u32)
                    .expect("writing to a String cannot fail");
            }
            ch => source.push(ch),
        }
    }
    source.push('"');
    source
}

fn legacy_float_source(value: f64) -> Option<String> {
    if !value.is_finite() {
        return None;
    }
    let mut source = value.to_string();
    if source.contains('e') || source.contains('E') {
        return None;
    }
    if !source.contains('.') {
        source.push_str(".0");
    }
    Some(source)
}

fn render_batch(values: &[String], render: fn(&[String]) -> String) -> usize {
    (0..LITERALS_PER_SAMPLE)
        .map(|_| black_box(render(black_box(values))).len())
        .sum()
}

fn measure<T>(run: impl FnOnce() -> T) -> Duration {
    let started = Instant::now();
    black_box(run());
    started.elapsed()
}

fn percentile_ns(samples: &mut [Duration], percentile: usize) -> u128 {
    samples.sort_unstable();
    let rank = (samples.len() * percentile).div_ceil(100);
    samples[rank.saturating_sub(1)].as_nanos()
}
