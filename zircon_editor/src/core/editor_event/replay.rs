use super::{EditorEventDispatcher, EditorEventRecord, EditorEventSource};
use thiserror::Error;

pub struct EditorEventReplay;

#[derive(Debug, Error)]
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
