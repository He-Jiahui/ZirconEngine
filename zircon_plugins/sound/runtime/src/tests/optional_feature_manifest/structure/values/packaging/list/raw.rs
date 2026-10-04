// 源码结构守卫：固定可选功能值转换的字段或值的归属；运行清单语义由 parity 测试验证。
use super::super::super::super::sources::*;

#[test]
fn optional_feature_packaging_list_maps_strings_through_raw_enum_parser() {
    assert!(
        PACKAGING_LIST.contains(".map(packaging_strategy_from_plugin_toml)"),
        "packaging enum-list projection should map strings through the raw enum parser"
    );
}
