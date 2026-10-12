use super::super::storage::DependencyParserState;

impl DependencyParserState {
    pub(in super::super::super) fn set_id(&mut self, id: String) {
        self.current_id = Some(id);
    }
}
