// 识别当前固定清单中的数组表边界；遇到下一张数组表即结束本行，单表语法不在该扫描器契约内。
use super::super::state::CapabilityStatusParserState;

pub(super) fn capability_status_table_transition_consumed(
    line: &str,
    parser: &mut CapabilityStatusParserState,
) -> bool {
    if line == "[[capability_statuses]]" {
        parser.begin_status_table();
        return true;
    }
    if line.starts_with("[[") {
        parser.leave_status_table();
    }
    !parser.is_inside_status()
}
