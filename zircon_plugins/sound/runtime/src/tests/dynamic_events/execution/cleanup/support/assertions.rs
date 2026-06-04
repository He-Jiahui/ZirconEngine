use super::super::super::super::*;

pub(crate) fn assert_next_execution_skipped_missing_executor(sound: &DefaultSoundManager) {
    let report = sound.execute_dynamic_events().unwrap();
    assert_eq!(report.executions.len(), 1);
    assert_eq!(
        report.executions[0].status,
        SoundDynamicEventExecutionStatus::SkippedMissingExecutor
    );
}
