// 静态主依赖与默认启用标志共享布尔字面量域；未知值应使对照测试失败。
pub(in super::super) fn bool_from_plugin_toml(value: &str) -> bool {
    match value {
        "true" => true,
        "false" => false,
        _ => panic!("unknown sound boolean value {value}"),
    }
}
