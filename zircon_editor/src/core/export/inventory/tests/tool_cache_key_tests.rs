use std::ffi::OsStr;
use std::hint::black_box;
use std::time::{Duration, Instant};

use super::tool_cache_key;

const SAMPLE_PAIRS: usize = 101;
const KEYS_PER_SAMPLE: usize = 4_096;

#[test]
fn editor896_tool_identity_cache_key_direct_preserves_exact_bytes() {
    for key in ["", "compiler", "工具🙂\0key"] {
        for program in [OsStr::new(""), OsStr::new("C:\\Tools\\工 具\\shader.exe")] {
            for args in [
                vec![],
                vec!["--version"],
                vec!["", "", "--print=中文", ""],
                vec!["line\nquoted\"", "nul\0between"],
            ] {
                assert_eq!(
                    tool_cache_key(key, program, &args),
                    legacy_key(key, program, &args)
                );
            }
        }
    }
}

#[test]
#[ignore = "release percentile evidence; run through the managed Windows validation lane"]
fn editor896_tool_identity_cache_key_direct_release_percentiles() {
    let program = OsStr::new("C:\\Program Files\\Zircon\\shader-compiler.exe");
    let args = (0..32)
        .map(|index| format!("--long-flag-{index:02}=some-value"))
        .collect::<Vec<_>>();
    let args = args.iter().map(String::as_str).collect::<Vec<_>>();
    let mut legacy = Vec::with_capacity(SAMPLE_PAIRS);
    let mut optimized = Vec::with_capacity(SAMPLE_PAIRS);
    for pair in 0..SAMPLE_PAIRS {
        if pair % 2 == 0 {
            legacy.push(measure(program, &args, legacy_key));
            optimized.push(measure(program, &args, tool_cache_key));
        } else {
            optimized.push(measure(program, &args, tool_cache_key));
            legacy.push(measure(program, &args, legacy_key));
        }
    }
    let legacy_p50_ns = percentile(&mut legacy.clone(), 50);
    let legacy_p95_ns = percentile(&mut legacy.clone(), 95);
    let legacy_p99_ns = percentile(&mut legacy, 99);
    let optimized_p50_ns = percentile(&mut optimized.clone(), 50);
    let optimized_p95_ns = percentile(&mut optimized.clone(), 95);
    let optimized_p99_ns = percentile(&mut optimized, 99);
    println!(
        "EDITOR896_TOOL_IDENTITY_CACHE_KEY_DIRECT_BENCH_V1 legacy_p50_ns={legacy_p50_ns} legacy_p95_ns={legacy_p95_ns} legacy_p99_ns={legacy_p99_ns} optimized_p50_ns={optimized_p50_ns} optimized_p95_ns={optimized_p95_ns} optimized_p99_ns={optimized_p99_ns}"
    );
    assert!(optimized_p95_ns <= legacy_p95_ns.saturating_mul(110).div_ceil(100));
}

fn legacy_key(key: &str, program: &OsStr, args: &[&str]) -> String {
    format!("{key}\0{program:?}\0{}", args.join("\0"))
}

fn measure(
    program: &OsStr,
    args: &[&str],
    cache_key: fn(&str, &OsStr, &[&str]) -> String,
) -> Duration {
    let started = Instant::now();
    let checksum = (0..KEYS_PER_SAMPLE)
        .map(|_| {
            black_box(cache_key(
                black_box("tool-key"),
                black_box(program),
                black_box(args),
            ))
            .len()
        })
        .sum::<usize>();
    black_box(checksum);
    started.elapsed()
}

fn percentile(samples: &mut [Duration], percent: usize) -> u128 {
    samples.sort_unstable();
    samples[(samples.len() * percent).div_ceil(100).saturating_sub(1)].as_nanos()
}
