// 仅在能力状态表内解释三项对照字段；其他清单字段由各自的测试投影处理。
mod bevy_references;
mod capability;
mod status;

use super::super::state::CapabilityStatusParserState;

pub(super) fn parse_capability_status_fields(line: &str, parser: &mut CapabilityStatusParserState) {
    if capability::parse_capability_field(line, parser) {
        return;
    }
    if status::parse_status_field(line, parser) {
        return;
    }
    bevy_references::parse_bevy_references_field(line, parser);
}
