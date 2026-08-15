#[test]
fn diagnostic_store_writer_streams_lines_without_an_intermediate_vec() {
    let source = include_str!("../../diagnostics.rs");
    let start = source
        .find("pub fn write_diagnostic_store_snapshot(")
        .expect("diagnostic store writer");
    let end = source[start..]
        .find("impl DiagnosticStoreLogSchedule")
        .map(|offset| start + offset)
        .expect("diagnostic store writer end");
    let writer_source = &source[start..end];

    assert!(
        !writer_source.contains("format_diagnostic_store_snapshot(snapshot)"),
        "the log writer must stream formatted series instead of collecting a second Vec"
    );
    assert!(
        writer_source.contains("write_log_lazy(scope"),
        "the log writer must defer series formatting until the process-log filter accepts it"
    );
    assert!(
        !writer_source.contains("write_log(scope"),
        "the diagnostic-store bridge must not eagerly format process-log messages"
    );
}

#[test]
fn allocation_heavy_process_log_producers_use_lazy_entry_points() {
    for (name, source) in [
        (
            "dynamic session construction",
            include_str!("../../../dynamic_api/session/construction.rs"),
        ),
        (
            "dynamic session project",
            include_str!("../../../dynamic_api/session/project.rs"),
        ),
        (
            "dynamic session state",
            include_str!("../../../dynamic_api/session/state.rs"),
        ),
        (
            "script scene system",
            include_str!("../../../script/vm/scene_system.rs"),
        ),
    ] {
        assert!(
            source.contains("write_log_lazy("),
            "{name} must use the lazy process-log entry point"
        );
        for eager_entry in [
            "write_diagnostic_log(",
            "write_debug_log(",
            "write_log(",
            "write_warn(",
            "write_error(",
            "write_diagnostic_log_at(",
        ] {
            let mut remaining = source;
            while let Some(start) = remaining.find(eager_entry) {
                let call = &remaining[start..];
                let end = call.find(");").unwrap_or(call.len());
                assert!(
                    !call[..end].contains("format!("),
                    "{name} must not format a process-log message before filtering"
                );
                remaining = &call[eager_entry.len()..];
            }
        }
    }
}
