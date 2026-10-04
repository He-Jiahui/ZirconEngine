// 表边界优先于字段解析，确保后续依赖和可选功能表的同名字段不并入能力状态。
mod fields;
mod table;

use super::state::CapabilityStatusParserState;

pub(super) fn parse_capability_status_line(line: &str, parser: &mut CapabilityStatusParserState) {
    if table::capability_status_table_transition_consumed(line, parser) {
        return;
    }
    fields::parse_capability_status_fields(line, parser);
}
