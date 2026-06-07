use super::super::storage::CapabilityStatusParserState;

impl CapabilityStatusParserState {
    pub(in super::super::super) fn set_bevy_references(&mut self, references: Vec<String>) {
        self.current_bevy_references = references;
    }
}
