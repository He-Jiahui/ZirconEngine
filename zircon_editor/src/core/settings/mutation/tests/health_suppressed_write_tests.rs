use super::*;

#[test]
fn skipped_or_invalid_project_write_never_marks_health_durable() {
    for terminal in [
        SettingsPersistenceTerminal::SkippedStale,
        SettingsPersistenceTerminal::BlockedInvalid,
        SettingsPersistenceTerminal::MissingWriteDisposition,
    ] {
        let health = SettingsPersistenceHealthAuthority::new(true);
        health.bind_project(7, true);
        let observation = health.begin_submission(
            SettingsDocumentIdentity::Project(7),
            SettingsFileGeneration::from_raw(11),
        );
        health.observe_terminal(observation, terminal);
        assert_eq!(
            health.snapshot().project().status(),
            SettingsPersistenceHealthStatus::Terminal(terminal)
        );
        assert!(!health.snapshot().project().status().is_retryable());
    }
}
