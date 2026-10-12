// 键集合从同一份完整静态选项投影取得，确保键测试和元数据测试不会各用一套解析规则。
pub(in crate::tests::manifest::support) fn option_keys_from_plugin_toml(
    manifest: &str,
) -> Vec<String> {
    super::super::parser::option_manifests_from_plugin_toml(manifest)
        .into_iter()
        .map(|option| option.key)
        .collect()
}
