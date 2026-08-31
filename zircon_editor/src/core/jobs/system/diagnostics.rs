use super::EditorJobSystem;

impl EditorJobSystem {
    pub fn event_journal_snapshot(&self) -> super::super::EditorJobEventJournalSnapshot {
        self.inner.event_queue.snapshot()
    }
}
