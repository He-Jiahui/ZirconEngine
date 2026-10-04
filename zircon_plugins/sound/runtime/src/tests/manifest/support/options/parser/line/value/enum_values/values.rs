// 枚举范围使用固定清单的简单字符串数组语法；结果由完整选项元数据测试比较。
use super::super::super::super::super::super::values::string_array_values;

pub(super) fn option_enum_values_from_plugin_toml(value: &str) -> Vec<String> {
    string_array_values(value)
}
