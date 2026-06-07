use super::super::storage::CapabilityStatusParserState;

impl CapabilityStatusParserState {
    pub(in super::super::super) fn set_capability(&mut self, capability: String) {
        self.current_capability = Some(capability);
    }
}
