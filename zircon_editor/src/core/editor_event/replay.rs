use super::{EditorEventDispatcher, EditorEventRecord, EditorEventSource};
use thiserror::Error;

/// 按记录顺序重新派发语义事件的回放入口；调用者需准备与原始执行相容的项目、插件及初始宿主状态。
pub struct EditorEventReplay;

#[derive(Debug, Error)]
/// 回放在首次成功/失败预期分歧处停止；原记录预期失败的分歧错误携带原序号，意外派发失败仅透传派发错误。
pub enum EditorEventReplayError<E>
where
    E: std::error::Error + 'static,
{
    #[error("replay expected event {sequence} to fail with {expected_error}, but it succeeded")]
    ExpectedFailureMissing {
        sequence: u64,
        expected_error: String,
    },
    #[error("replay expected event {sequence} to fail with {expected_error}, but got {actual}")]
    UnexpectedFailure {
        sequence: u64,
        expected_error: String,
        #[source]
        actual: E,
    },
    #[error(transparent)]
    Dispatch(#[from] E),
}

impl EditorEventReplay {
    /// 重新执行事件并在首个错误预期分歧处停止；原记录的结果、修订号和副作用不会直接恢复到宿主。
    /// 失败记录目前按错误文本比较，成功记录只检查派发成功；结果值与最终状态需由调用方另行验证。
    pub fn replay<D>(
        runtime: &D,
        records: &[EditorEventRecord],
    ) -> Result<(), EditorEventReplayError<D::Error>>
    where
        D: EditorEventDispatcher,
    {
        for record in records {
            match (
                runtime.dispatch_event(EditorEventSource::Replay, record.event.clone()),
                record.result.error.as_ref(),
            ) {
                (Ok(_), None) => {}
                (Ok(_), Some(expected_error)) => {
                    return Err(EditorEventReplayError::ExpectedFailureMissing {
                        sequence: record.sequence.0,
                        expected_error: expected_error.clone(),
                    });
                }
                (Err(error), Some(expected_error))
                    if error.to_string() == expected_error.as_str() => {}
                (Err(error), Some(expected_error)) => {
                    return Err(EditorEventReplayError::UnexpectedFailure {
                        sequence: record.sequence.0,
                        expected_error: expected_error.clone(),
                        actual: error,
                    });
                }
                (Err(error), None) => return Err(EditorEventReplayError::Dispatch(error)),
            }
        }
        Ok(())
    }
}
