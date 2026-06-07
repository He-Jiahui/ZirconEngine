use super::super::super::super::StaticDependency;
use super::super::storage::DependencyParserState;

impl DependencyParserState {
    pub(in super::super::super) fn finish(mut self) -> Vec<StaticDependency> {
        self.push_current_dependency();
        self.dependencies
    }
}
