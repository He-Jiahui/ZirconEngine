// 将静态及运行时选项投影为同一比较域，包含默认值、枚举范围及所需能力。
pub(in crate::tests::manifest::support) fn option_manifest_tuple(
    option: zircon_runtime::plugin::PluginOptionManifest,
) -> (String, String, String, String, Vec<String>, Option<String>) {
    (
        option.key,
        option.display_name,
        option.value_type,
        option.default_value,
        option.enum_values,
        option.required_capability,
    )
}
