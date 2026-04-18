use super::{EditorEventRecord, EditorEventRuntime, EditorEventSource};

pub struct EditorEventReplay;

impl EditorEventReplay {
    pub fn replay(
        runtime: &EditorEventRuntime,
        records: &[EditorEventRecord],
    ) -> Result<(), String> {
        for record in records {
            runtime.dispatch_event(EditorEventSource::Replay, record.event.clone())?;
        }
        Ok(())
    }
}
