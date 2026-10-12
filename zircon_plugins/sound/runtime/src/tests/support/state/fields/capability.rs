use super::super::storage::DependencyParserState;

impl DependencyParserState {
    pub(in super::super::super) fn set_capability(&mut self, capability: String) {
        self.current_capability = Some(capability);
    }
}
