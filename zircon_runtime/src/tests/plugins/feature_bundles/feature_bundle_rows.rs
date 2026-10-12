//! 通用特性遍历只提供行及诊断上下文，未声明或非数组会跳过；字段结构是否允许由独立模式与包种类测试检查。
use std::path::Path;

pub(super) fn for_each_feature_bundle(
    table: &toml::Table,
    relative_path: &Path,
    field_name: &str,
    row_label: &str,
    visit: &mut impl FnMut(&toml::Table, &str),
) {
    let Some(features) = table.get(field_name).and_then(toml::Value::as_array) else {
        return;
    };

    for feature in features {
        let feature_table = feature.as_table().unwrap_or_else(|| {
            panic!("plugin manifest {relative_path:?} {row_label} should be a table")
        });
        let feature_id = feature_table
            .get("id")
            .and_then(toml::Value::as_str)
            .unwrap_or("<unknown>");
        let context = format!("{row_label} `{feature_id}`");
        visit(feature_table, &context);
    }
}
