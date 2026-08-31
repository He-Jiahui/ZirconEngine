use std::collections::BTreeMap;
use std::hint::black_box;
use std::time::Instant;

const PERF_MARKER: &str = "RUNTIME358_UI_STYLE_SHEET_CAPACITY_BENCH_V1";

#[test]
fn optimization_batch_20260830bf_runtime_style_resolver_reserves_total_sheet_capacity() {
    let source = include_str!("../ui_style_resolver.rs");
    assert!(source.contains("Vec::with_capacity("));
    assert!(source.contains("imported_stylesheet_count"));
    assert!(source.contains("sheets.extend_from_slice(artifacts.widget_styles())"));
    assert_eq!(
        source
            .matches("compiler.style_imports.get(reference)")
            .count(),
        1
    );
    assert!(!source.contains("artifacts.widget_styles().to_vec()"));
}

#[test]
fn optimization_batch_20260830bf_runtime_style_resolver_keeps_sheet_order() {
    let source = include_str!("../ui_style_resolver.rs");
    let widgets = source
        .find("sheets.extend_from_slice(artifacts.widget_styles())")
        .expect("widget styles append");
    let imports = source
        .find("let imported_styles = document")
        .expect("imported styles resolution");
    let imported_append = source
        .find("for imported in imported_styles")
        .expect("imported styles append");
    let local = source
        .find("for stylesheet in &document.stylesheets")
        .expect("local styles loop");
    assert!(widgets < imports && imports < imported_append && imported_append < local);
}

#[test]
#[ignore = "release performance evidence"]
fn optimization_batch_20260830bf_runtime_style_resolver_capacity_p95() {
    const SHEETS: usize = 2_048;
    const IMPORTS: usize = 64;
    const STYLES_PER_IMPORT: usize = SHEETS / IMPORTS;
    const BUILDS: usize = 2_048;
    const SAMPLES: usize = 17;
    let imports = (0..IMPORTS)
        .map(|index| format!("style-{index}"))
        .collect::<Vec<_>>();
    let imported = (0..STYLES_PER_IMPORT).collect::<Vec<_>>();
    let style_imports = imports
        .iter()
        .cloned()
        .map(|reference| (reference, imported.clone()))
        .collect::<BTreeMap<_, _>>();
    let widget_styles = [usize::MAX];
    let local_styles = [usize::MIN];
    let mut baseline = Vec::with_capacity(SAMPLES);
    let mut candidate = Vec::with_capacity(SAMPLES);
    for sample in 0..SAMPLES {
        let order = if sample % 2 == 0 { [0, 1] } else { [1, 0] };
        for pass in order {
            let started = Instant::now();
            let mut checksum = 0usize;
            for _ in 0..BUILDS {
                let mut values = if pass == 0 {
                    Vec::new()
                } else {
                    let resolved = imports
                        .iter()
                        .filter_map(|reference| style_imports.get(reference))
                        .collect::<Vec<_>>();
                    let imported_count = resolved.iter().map(|styles| styles.len()).sum();
                    let mut values = Vec::with_capacity(
                        widget_styles.len() + imported_count + local_styles.len(),
                    );
                    values.extend_from_slice(&widget_styles);
                    for styles in resolved {
                        values.extend(styles.iter().copied());
                    }
                    values.extend_from_slice(&local_styles);
                    checksum = checksum.wrapping_add(values.len());
                    black_box(values);
                    continue;
                };
                let imported_count = imports
                    .iter()
                    .filter_map(|reference| style_imports.get(reference))
                    .map(|styles| styles.len())
                    .sum::<usize>();
                values.extend_from_slice(&widget_styles);
                for reference in &imports {
                    if let Some(styles) = style_imports.get(reference) {
                        values.extend(styles.iter().copied());
                    }
                }
                values.extend_from_slice(&local_styles);
                debug_assert_eq!(imported_count, SHEETS);
                checksum = checksum.wrapping_add(values.len());
                black_box(values);
            }
            black_box(checksum);
            let elapsed = started.elapsed().as_nanos();
            if pass == 0 {
                baseline.push(elapsed);
            } else {
                candidate.push(elapsed);
            }
        }
    }
    baseline.sort_unstable();
    candidate.sort_unstable();
    let baseline_p95 = baseline[(SAMPLES * 95).div_ceil(100) - 1];
    let candidate_p95 = candidate[(SAMPLES * 95).div_ceil(100) - 1];
    let reduction =
        100.0 * baseline_p95.saturating_sub(candidate_p95) as f64 / baseline_p95.max(1) as f64;
    println!(
        "{PERF_MARKER} sheets={SHEETS} builds={BUILDS} samples={SAMPLES} baseline_p95_ns={baseline_p95} candidate_p95_ns={candidate_p95} p95_reduction_percent={reduction:.2}"
    );
    assert!(candidate_p95.saturating_mul(10) <= baseline_p95.saturating_mul(7));
}
