#[test]
fn process_bins_install_crash_flush_and_shutdown_after_entry_runner_returns() {
    for (name, source) in [
        ("editor", include_str!("../src/bin/editor.rs")),
        (
            "runtime_preview",
            include_str!("../src/bin/runtime_preview.rs"),
        ),
    ] {
        let install = source
            .find("install_process_log_panic_flush(DEFAULT_DIAGNOSTIC_LOG_CRASH_FLUSH_TIMEOUT)")
            .unwrap_or_else(|| panic!("{name} bounded panic flush installation"));
        let run = source
            .find("let result = zircon_app::EntryRunner::run_")
            .unwrap_or_else(|| panic!("{name} captured entry runner result"));
        let shutdown = source
            .rfind("shutdown_process_log(DEFAULT_DIAGNOSTIC_LOG_SHUTDOWN_TIMEOUT)")
            .unwrap_or_else(|| panic!("{name} process-log shutdown"));
        assert!(
            install < run && run < shutdown,
            "{name} must install before entry and drain after entry returns"
        );
        assert!(
            !source[run..shutdown].contains('?'),
            "{name} must not return early before process-log shutdown"
        );
        assert!(
            source[shutdown..].contains("result"),
            "{name} must return or match the captured result after shutdown"
        );
        assert!(source.contains("DEFAULT_DIAGNOSTIC_LOG_SHUTDOWN_TIMEOUT"));
    }
}
