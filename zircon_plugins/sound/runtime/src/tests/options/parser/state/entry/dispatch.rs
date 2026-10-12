use super::super::super::line::parse_option_manifest_line;
use super::super::storage::OptionManifestParserState;

impl OptionManifestParserState {
    pub(in super::super::super) fn parse_manifest_line(&mut self, line: &str) {
        if self.consume_option_table_transition(line) {
            return;
        }
        parse_option_manifest_line(line, &mut self.pending);
    }
}
