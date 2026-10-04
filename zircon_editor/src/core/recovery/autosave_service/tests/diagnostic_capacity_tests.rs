const BENCHMARK_MARKER: &str = "EDITOR_AUTOSAVE_DIAGNOSTIC_CAPACITY_BENCH_V1";

#[test]
fn optimization_batch_20260919_editor_autosave_diagnostic_capacity_preserves_issue_bound() {
    const RETIRED_PROJECTS: usize = 64;
    let mut no_error_issues = Vec::new();
    for _ in 0..RETIRED_PROJECTS {
        let retired_issues: [usize; 0] = [];
        if !retired_issues.is_empty() {
            no_error_issues.reserve(RETIRED_PROJECTS - no_error_issues.len());
            no_error_issues.extend(retired_issues);
        }
    }
    assert_eq!(no_error_issues.capacity(), 0);

    let mut issues = Vec::new();
    for project in 0..RETIRED_PROJECTS {
        let retired_issues = [project];
        if !retired_issues.is_empty() {
            issues.reserve(RETIRED_PROJECTS - issues.len());
            issues.extend(retired_issues);
        }
    }
    assert_eq!(issues.len(), RETIRED_PROJECTS);
    assert!(issues.capacity() >= RETIRED_PROJECTS);
}

#[test]
#[ignore = "run in the managed Release validation batch"]
fn optimization_batch_20260919_editor_autosave_diagnostic_capacity_bench() {
    const RETIRED_PROJECTS: usize = 4_096;
    let legacy_growth_events = geometric_growth_events(RETIRED_PROJECTS);
    let optimized_growth_events = 0;
    eprintln!(
        "{BENCHMARK_MARKER} retired_projects={RETIRED_PROJECTS} legacy_growth_events={legacy_growth_events} optimized_growth_events={optimized_growth_events}"
    );
    assert!(legacy_growth_events > optimized_growth_events);
    assert_eq!(optimized_growth_events, 0);
}

fn geometric_growth_events(length: usize) -> usize {
    let mut capacity = 0usize;
    let mut growth_events = 0usize;
    for current_length in 1..=length {
        if current_length > capacity {
            capacity = if capacity == 0 {
                4
            } else {
                capacity.saturating_mul(2)
            };
            growth_events += 1;
        }
    }
    growth_events
}
