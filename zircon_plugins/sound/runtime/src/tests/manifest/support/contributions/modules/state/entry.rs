use super::super::line::parse_module_contribution_line;
use super::storage::ModuleContributionParserState;

impl ModuleContributionParserState {
    pub(in super::super) fn parse_manifest_line(&mut self, line: &str) {
        if self.module_table_transition_consumed(line) {
            return;
        }
        parse_module_contribution_line(
            line,
            &mut self.current_name,
            &mut self.current_kind,
            &mut self.current_crate_name,
            &mut self.current_target_modes,
            &mut self.current_capabilities,
        );
    }
}
