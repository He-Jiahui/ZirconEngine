// 值类型、默认值与枚举域必须进入同一选项，供完整元数据对照发现声明分叉。
use super::super::super::super::state::PendingOptionManifest;
use super::{default_value, enum_values, value_type};

pub(in crate::tests::manifest::support::options::parser::line) fn parse_option_value_line(
    line: &str,
    pending: &mut PendingOptionManifest,
) -> bool {
    if value_type::parse_option_value_type_line(line, pending) {
        return true;
    }
    if default_value::parse_option_default_value_line(line, pending) {
        return true;
    }
    enum_values::parse_option_enum_values_line(line, pending)
}
