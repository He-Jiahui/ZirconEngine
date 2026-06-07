use super::storage::DependencyParserState;

impl DependencyParserState {
    pub(in super::super) fn begin_dependency_table(&mut self) {
        self.push_current_dependency();
        self.inside_dependency = true;
    }

    pub(in super::super) fn leave_dependency_table(&mut self) {
        self.push_current_dependency();
        self.inside_dependency = false;
    }

    pub(in super::super) fn is_inside_dependency(&self) -> bool {
        self.inside_dependency
    }
}
