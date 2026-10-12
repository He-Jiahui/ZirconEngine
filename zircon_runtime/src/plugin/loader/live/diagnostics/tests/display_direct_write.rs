use std::fmt::Write as _;
use std::hint::black_box;
use std::time::{Duration, Instant};

use super::NativePluginBehaviorDiagnosticError;

const SAMPLE_PAIRS: usize = 101;
const DISPLAYS_PER_SAMPLE: usize = 4_096;
const DIAGNOSTICS_PER_ERROR: usize = 64;

// 将 Display 的直接写入结果与旧 join 基线逐字节比较，覆盖空列表、空项及中文和 emoji 诊断。
#[test]
fn runtime874_native_diagnostic_display_direct_write_preserves_exact_text() {
    for diagnostics in [
        Vec::new(),
        vec!["only".to_string()],
        vec!["".to_string(), "middle".to_string(), "".to_string()],
        vec!["插件加载失败".to_string(), "emoji🙂".to_string()],
    ] {
        let error = failed_status(diagnostics);
        assert_eq!(error.to_string(), legacy_display(&error));
    }
}

// 交替测量顺序减轻固定先后带来的偏差；性能门槛要求直接写入路径的 P95 不超过旧路径的 110%。
#[test]
#[ignore = "release percentile evidence; run through the managed Windows validation lane"]
fn runtime874_native_diagnostic_display_direct_write_release_percentiles() {
    let error = failed_status(
        (0..DIAGNOSTICS_PER_ERROR)
            .map(|index| format!("native diagnostic {index:02}: {}", "x".repeat(24)))
            .collect(),
    );
    let mut legacy_samples = Vec::with_capacity(SAMPLE_PAIRS);
    let mut optimized_samples = Vec::with_capacity(SAMPLE_PAIRS);

    for pair in 0..SAMPLE_PAIRS {
        if pair % 2 == 0 {
            legacy_samples.push(measure(|| render_batch(&error, legacy_display)));
            optimized_samples.push(measure(|| render_batch(&error, direct_display)));
        } else {
            optimized_samples.push(measure(|| render_batch(&error, direct_display)));
            legacy_samples.push(measure(|| render_batch(&error, legacy_display)));
        }
    }

    let legacy_p50_ns = percentile_ns(&mut legacy_samples.clone(), 50);
    let legacy_p95_ns = percentile_ns(&mut legacy_samples.clone(), 95);
    let legacy_p99_ns = percentile_ns(&mut legacy_samples, 99);
    let optimized_p50_ns = percentile_ns(&mut optimized_samples.clone(), 50);
    let optimized_p95_ns = percentile_ns(&mut optimized_samples.clone(), 95);
    let optimized_p99_ns = percentile_ns(&mut optimized_samples, 99);
    println!(
        "RUNTIME874_NATIVE_DIAGNOSTIC_DISPLAY_DIRECT_WRITE_BENCH_V1 \
legacy_p50_ns={legacy_p50_ns} legacy_p95_ns={legacy_p95_ns} \
legacy_p99_ns={legacy_p99_ns} optimized_p50_ns={optimized_p50_ns} \
optimized_p95_ns={optimized_p95_ns} optimized_p99_ns={optimized_p99_ns}"
    );
    assert!(
        optimized_p95_ns <= legacy_p95_ns.saturating_mul(110).div_ceil(100),
        "direct-write p95 regressed beyond 10%: legacy={legacy_p95_ns}ns optimized={optimized_p95_ns}ns"
    );
}

fn failed_status(diagnostics: Vec<String>) -> NativePluginBehaviorDiagnosticError {
    NativePluginBehaviorDiagnosticError::FailedStatus {
        label: "native behavior".to_string(),
        status_code: 1,
        diagnostics,
    }
}

fn display_capacity(error: &NativePluginBehaviorDiagnosticError) -> usize {
    match error {
        NativePluginBehaviorDiagnosticError::FailedStatus { diagnostics, .. } => diagnostics
            .iter()
            .map(String::len)
            .sum::<usize>()
            .saturating_add(diagnostics.len().saturating_sub(1).saturating_mul(2)),
    }
}

fn direct_display(error: &NativePluginBehaviorDiagnosticError) -> String {
    let mut output = String::with_capacity(display_capacity(error));
    write!(&mut output, "{error}").expect("writing to String cannot fail");
    output
}

fn legacy_display(error: &NativePluginBehaviorDiagnosticError) -> String {
    let NativePluginBehaviorDiagnosticError::FailedStatus { diagnostics, .. } = error;
    let joined = diagnostics.join("; ");
    let mut output = String::with_capacity(display_capacity(error));
    output.push_str(&joined);
    output
}

fn render_batch(
    error: &NativePluginBehaviorDiagnosticError,
    render: fn(&NativePluginBehaviorDiagnosticError) -> String,
) -> usize {
    (0..DISPLAYS_PER_SAMPLE)
        .map(|_| black_box(render(black_box(error))).len())
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
