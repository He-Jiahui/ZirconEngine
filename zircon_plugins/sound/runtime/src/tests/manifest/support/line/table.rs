// 当前固定清单用数组表界定记录；切换到其他表时完成上一条，避免相同字段跨表串入。
use super::super::state::DependencyParserState;

pub(super) fn dependency_table_transition_consumed(
    line: &str,
    parser: &mut DependencyParserState,
) -> bool {
    if line == "[[dependencies]]" {
        parser.begin_dependency_table();
        return true;
    }
    if line.starts_with("[[") {
        parser.leave_dependency_table();
    }
    !parser.is_inside_dependency()
}
