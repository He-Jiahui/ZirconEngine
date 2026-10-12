// 先判定表头并完成状态切换，再把普通行交给当前区段；避免跨表字段污染。
use super::super::super::section::OptionalFeatureSection;
use super::super::{section_line, transition, OptionalFeatureParserState};

impl OptionalFeatureParserState {
    pub(in super::super::super) fn parse_manifest_line(&mut self, line: &str) {
        if let Some(section) = OptionalFeatureSection::from_table_header(line) {
            transition::enter_section(self, section);
            return;
        }

        section_line::parse_section_line(self, line);
    }
}
