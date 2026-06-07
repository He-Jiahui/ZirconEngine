use super::super::storage::OptionManifestParserState;

impl OptionManifestParserState {
    pub(in super::super::super) fn consume_option_table_transition(&mut self, line: &str) -> bool {
        if line == "[[options]]" {
            self.pending.push_into(&mut self.options);
            self.inside_option = true;
            return true;
        }
        if line.starts_with("[[") {
            self.pending.push_into(&mut self.options);
            self.inside_option = false;
        }
        !self.inside_option
    }
}
