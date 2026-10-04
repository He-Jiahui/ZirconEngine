// 静态依赖和选项扫描器共享布尔值域，未知字面量必须使清单对照失败。
pub(super) fn bool_from_plugin_toml(value: &str) -> bool {
    match value {
        "true" => true,
        "false" => false,
        _ => panic!("unknown sound boolean value {value}"),
    }
}
