// 源码结构守卫：固定可选功能值转换的字段或值的归属；运行清单语义由 parity 测试验证。
use super::super::super::sources::*;

#[test]
fn optional_feature_string_raw_parser_owner_stays_isolated() {
    assert!(
        STRING_RAW.contains("value.to_string()"),
        "string raw TOML parser must remain isolated in raw.rs"
    );
}
