// 源码结构守卫：固定可选功能值转换的模块职责和重导出边界；运行清单语义由 parity 测试验证。
use super::super::super::super::sources::*;

#[test]
fn optional_feature_string_facade_does_not_reexport_raw_parser() {
    assert!(
        !STRING_ROOT.contains("use raw::string_from_plugin_toml"),
        "string parent must not re-export the raw TOML parser"
    );
}
