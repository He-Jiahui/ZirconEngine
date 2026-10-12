// 消耗一个已排队事件并核对执行器缺失状态；清理用例须先提交事件，避免空队列掩盖残留执行器。
use super::super::super::super::*;

pub(crate) fn assert_next_execution_skipped_missing_executor(sound: &DefaultSoundManager) {
    let report = sound.execute_dynamic_events().unwrap();
    assert_eq!(report.executions.len(), 1);
    assert_eq!(
        report.executions[0].status,
        SoundDynamicEventExecutionStatus::SkippedMissingExecutor
    );
}
