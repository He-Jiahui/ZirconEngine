use super::super::storage::DependencyParserState;

impl DependencyParserState {
    pub(in super::super::super) fn set_required(&mut self, required: bool) {
        self.current_required = Some(required);
    }
}
