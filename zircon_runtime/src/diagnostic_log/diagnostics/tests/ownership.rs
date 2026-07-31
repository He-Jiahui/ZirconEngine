#[test]
fn diagnostic_log_snapshot_bridge_stays_single_owner() {
    let manifest_dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    let diagnostic_log_dir = manifest_dir.join("src").join("diagnostic_log");
    let core_diagnostics_dir = manifest_dir
        .join("src")
        .join("core")
        .join("runtime")
        .join("diagnostics");

    for entry in std::fs::read_dir(&diagnostic_log_dir)
        .unwrap_or_else(|error| panic!("failed to read {}: {error}", diagnostic_log_dir.display()))
    {
        let entry =
            entry.unwrap_or_else(|error| panic!("failed to read diagnostic_log entry: {error}"));
        let path = entry.path();
        if path.extension().and_then(|value| value.to_str()) != Some("rs")
            || path.file_name().and_then(|value| value.to_str()) == Some("diagnostics.rs")
        {
            continue;
        }
        let source = std::fs::read_to_string(&path)
            .unwrap_or_else(|error| panic!("failed to read {}: {error}", path.display()));
        for forbidden_bridge_token in [
            "core::diagnostics",
            "DiagnosticStoreSnapshot",
            "DiagnosticSeriesSnapshot",
        ] {
            assert!(
                !source.contains(forbidden_bridge_token),
                "diagnostic_log file {} should not bypass diagnostics.rs with `{forbidden_bridge_token}`",
                path.display()
            );
        }
    }

    let mut core_diagnostic_files = Vec::new();
    collect_rust_files(&core_diagnostics_dir, &mut core_diagnostic_files);
    for path in core_diagnostic_files {
        let source = std::fs::read_to_string(&path)
            .unwrap_or_else(|error| panic!("failed to read {}: {error}", path.display()));
        for forbidden_log_token in [
            "diagnostic_log",
            "write_log",
            "write_diagnostic",
            "initialize_process_log",
            "DiagnosticLog",
        ] {
            assert!(
                !source.contains(forbidden_log_token),
                "core diagnostics file {} should not depend on process-log token `{forbidden_log_token}`",
                path.display()
            );
        }
    }
}

fn collect_rust_files(root: &std::path::Path, files: &mut Vec<std::path::PathBuf>) {
    for entry in std::fs::read_dir(root)
        .unwrap_or_else(|error| panic!("failed to read {}: {error}", root.display()))
    {
        let entry =
            entry.unwrap_or_else(|error| panic!("failed to read diagnostics entry: {error}"));
        let path = entry.path();
        if path.is_dir() {
            collect_rust_files(&path, files);
            continue;
        }
        if path.extension().and_then(|value| value.to_str()) == Some("rs") {
            files.push(path);
        }
    }
}
