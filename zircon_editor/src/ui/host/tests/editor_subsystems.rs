use super::{
    editor_runtime_sandbox_enabled_from_config, editor_subsystem_report_from_config,
    EditorSubsystemReport, EDITOR_SUBSYSTEM_ANIMATION_AUTHORING,
    EDITOR_SUBSYSTEM_NATIVE_WINDOW_HOSTING, OPTIONAL_EDITOR_SUBSYSTEMS,
};

#[test]
fn subsystem_report_projects_known_and_custom_configured_capabilities() {
    let report = editor_subsystem_report_from_config(Some(vec![
        EDITOR_SUBSYSTEM_ANIMATION_AUTHORING.to_string(),
        "editor.extension.custom_fixture".to_string(),
    ]));

    assert!(report.is_enabled(EDITOR_SUBSYSTEM_ANIMATION_AUTHORING));
    assert!(report.is_enabled("editor.extension.custom_fixture"));
    assert!(!report.is_enabled(EDITOR_SUBSYSTEM_NATIVE_WINDOW_HOSTING));
    assert_eq!(
        report.diagnostics(),
        ["custom editor capability enabled: editor.extension.custom_fixture"]
    );
}

#[test]
fn absent_subsystem_configuration_keeps_the_default_enabled_set() {
    let report = editor_subsystem_report_from_config(None);

    assert_eq!(report, EditorSubsystemReport::default_enabled());
}

#[test]
fn optimization_wave_20260825vw_editor52_default_capabilities_are_sorted_for_lookup() {
    let report = EditorSubsystemReport::default_enabled();
    let mut expected = OPTIONAL_EDITOR_SUBSYSTEMS
        .iter()
        .map(|item| (*item).to_string())
        .collect::<Vec<_>>();
    expected.sort_unstable();

    assert_eq!(report.enabled_subsystems(), expected);
    assert!(expected
        .iter()
        .all(|capability| report.is_enabled(capability)));
    assert!(!report.is_enabled("editor.extension.not-installed"));
}

#[test]
#[ignore = "release-only performance evidence"]
fn optimization_wave_20260825vw_editor52_capability_binary_lookup_evidence() {
    const CAPABILITY_COUNT: usize = 10_000;
    const QUERY_COUNT: usize = 100_000;
    const TARGET_MILLIS: u128 = 500;
    const MARKER: &str = "EDITOR52_CAPABILITY_BINARY_LOOKUP_BENCH_V1";

    let requested = (0..CAPABILITY_COUNT)
        .map(|index| format!("editor.extension.zzzz-{index:05}"))
        .collect::<Vec<_>>();
    let target = requested.last().expect("last generated capability").clone();
    let report = editor_subsystem_report_from_config(Some(requested));

    let started = std::time::Instant::now();
    for _ in 0..QUERY_COUNT {
        assert!(std::hint::black_box(
            report.is_enabled(std::hint::black_box(&target))
        ));
    }
    let elapsed = started.elapsed();
    let enabled_count = report.enabled_subsystems().len();
    let legacy_comparisons = enabled_count * QUERY_COUNT;
    let indexed_probe_upper_bound =
        (usize::BITS - enabled_count.leading_zeros()) as usize * QUERY_COUNT;

    assert!(
        elapsed.as_millis() <= TARGET_MILLIS,
        "{MARKER} elapsed_ms={} target_ms={TARGET_MILLIS}",
        elapsed.as_millis()
    );
    println!(
        "{MARKER} capabilities={enabled_count} queries={QUERY_COUNT} legacy_comparisons={legacy_comparisons} indexed_probe_upper_bound={indexed_probe_upper_bound} reduction_pct=99.86 elapsed_ms={} target_ms={TARGET_MILLIS}",
        elapsed.as_millis()
    );
}

#[test]
fn absent_runtime_sandbox_configuration_keeps_the_safe_default() {
    assert!(editor_runtime_sandbox_enabled_from_config(None));
    assert!(!editor_runtime_sandbox_enabled_from_config(Some(false)));
}
