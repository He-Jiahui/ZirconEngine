use super::super::super::super::StaticModule;
use super::super::storage::ModuleContributionParserState;

impl ModuleContributionParserState {
    pub(in super::super::super) fn finish(mut self) -> Vec<StaticModule> {
        self.push_current_module();
        self.modules
    }
}
