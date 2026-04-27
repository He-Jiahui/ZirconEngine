use super::{EditorEventRecord, EditorEventRuntime, EditorEventSource};

pub struct EditorEventReplay;

impl EditorEventReplay {
    pub fn replay(
        runtime: &EditorEventRuntime,
        records: &[EditorEventRecord],
    ) -> Result<(), String> {
        for record in records {
            match (
                runtime.dispatch_event(EditorEventSource::Replay, record.event.clone()),
                record.result.error.as_ref(),
            ) {
                (Ok(_), None) => {}
                (Ok(_), Some(expected_error)) => {
                    return Err(format!(
                        "replay expected event {} to fail with {expected_error}, but it succeeded",
                        record.sequence.0
                    ));
                }
                (Err(error), Some(expected_error)) if &error == expected_error => {}
                (Err(error), Some(expected_error)) => {
                    return Err(format!(
                        "replay expected event {} to fail with {expected_error}, but got {error}",
                        record.sequence.0
                    ));
                }
                (Err(error), None) => return Err(error),
            }
        }
        Ok(())
    }
}
