use std::hint::black_box;
use std::time::Instant;

use zircon_runtime_interface::ui::style::{UiStyleColor, UiThemeTokenRef};

use super::{normalized_theme_role, UiThemeRegistry};

const SAMPLE_PAIRS: usize = 31;
const RESOLUTIONS_PER_SAMPLE: usize = 100_000;
const THEME_ROLE: &str = "$theme.palette.surface.3";

#[test]
fn optimization_batch_20260828iu_runtime293_role_prefixes_preserve_token_resolution() {
    let registry = UiThemeRegistry::default();
    for role in [
        "palette.surface.3",
        "theme.palette.surface.3",
        "theme:palette.surface.3",
        "$theme.palette.surface.3",
    ] {
        assert_eq!(
            registry.resolve_role(role),
            legacy_resolve_role(&registry, role)
        );
    }
    assert!(registry.resolve_role("theme.palette.unknown").is_none());
}

#[test]
fn optimization_batch_20260828iu_runtime293_role_resolution_borrows_token_name() {
    let source = include_str!("mod.rs");
    let implementation = source.split("#[cfg(test)]").next().expect("implementation");
    let role_body = implementation
        .split("pub fn resolve_role")
        .nth(1)
        .and_then(|body| body.split("pub fn resolve_style_color").next())
        .expect("role resolver");

    assert!(role_body.contains("self.resolve_token_name(token)"));
    assert!(!role_body.contains("UiThemeTokenRef::new"));
    assert!(implementation.contains("fn resolve_token_name(&self, token: &str)"));
}

#[test]
#[ignore = "managed Windows release performance evidence"]
fn optimization_batch_20260828iu_runtime293_borrowed_theme_role_resolution_bench() {
    let registry = UiThemeRegistry::default();
    let mut legacy_samples = Vec::with_capacity(SAMPLE_PAIRS);
    let mut optimized_samples = Vec::with_capacity(SAMPLE_PAIRS);
    for pair in 0..SAMPLE_PAIRS {
        if pair % 2 == 0 {
            legacy_samples.push(measure(&registry, false));
            optimized_samples.push(measure(&registry, true));
        } else {
            optimized_samples.push(measure(&registry, true));
            legacy_samples.push(measure(&registry, false));
        }
    }
    let legacy_p50_ns = percentile(&legacy_samples, 50);
    let optimized_p50_ns = percentile(&optimized_samples, 50);
    let legacy_p95_ns = percentile(&legacy_samples, 95);
    let optimized_p95_ns = percentile(&optimized_samples, 95);
    println!(
        "RUNTIME293_BORROWED_THEME_ROLE_RESOLUTION_BENCH_V1 sample_pairs={SAMPLE_PAIRS} \
resolutions_per_sample={RESOLUTIONS_PER_SAMPLE} role_bytes={} \
legacy_token_allocations_per_resolution=1 optimized_token_allocations_per_resolution=0 \
legacy_p50_ns={legacy_p50_ns} optimized_p50_ns={optimized_p50_ns} \
legacy_p95_ns={legacy_p95_ns} optimized_p95_ns={optimized_p95_ns} \
legacy_raw_ns={} optimized_raw_ns={}",
        THEME_ROLE.len(),
        sample_csv(&legacy_samples),
        sample_csv(&optimized_samples),
    );
    assert!(optimized_p95_ns.saturating_mul(100) <= legacy_p95_ns.saturating_mul(70));
}

fn legacy_resolve_role(registry: &UiThemeRegistry, role: &str) -> Option<UiStyleColor> {
    let token = normalized_theme_role(role)?;
    registry.resolve_token(&UiThemeTokenRef::new(token))
}

fn measure(registry: &UiThemeRegistry, optimized: bool) -> u128 {
    let started = Instant::now();
    for _ in 0..RESOLUTIONS_PER_SAMPLE {
        let resolved = if optimized {
            registry.resolve_role(black_box(THEME_ROLE))
        } else {
            legacy_resolve_role(registry, black_box(THEME_ROLE))
        };
        black_box(resolved);
    }
    started.elapsed().as_nanos().max(1)
}

fn percentile(samples: &[u128], percentile: usize) -> u128 {
    let mut sorted = samples.to_vec();
    sorted.sort_unstable();
    let rank = (sorted.len() * percentile).div_ceil(100);
    sorted[rank.saturating_sub(1)]
}

fn sample_csv(samples: &[u128]) -> String {
    samples
        .iter()
        .map(u128::to_string)
        .collect::<Vec<_>>()
        .join(",")
}
