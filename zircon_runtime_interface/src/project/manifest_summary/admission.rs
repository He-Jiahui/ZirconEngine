use super::{
    ProjectManifestSummaryError, MAX_PROJECT_MANIFEST_ARRAY_ITEMS,
    MAX_PROJECT_MANIFEST_NESTING_DEPTH, MAX_PROJECT_MANIFEST_TABLE_ENTRIES,
};

/// 在 TOML 已解析后、转为 JSON 值之前约束容器深度和元素总量。
/// 此处不限制原始文档字节数，入口调用方须先执行字节预算。
///
/// # 参数
/// root 是已由 TOML 解析器构造的值树；根表从深度 0 开始。
///
/// # 返回值
/// 表项和数组元素分别按整棵树累计；容器层级超过预算或累计数量超限时返回对应错误。
///
pub(super) fn validate_toml_complexity(
    root: &toml::Value,
) -> Result<(), ProjectManifestSummaryError> {
    let mut pending = Vec::with_capacity(1);
    pending.push((root, 0_usize));
    let mut table_entries = 0_usize;
    let mut array_items = 0_usize;

    while let Some((value, depth)) = pending.pop() {
        match value {
            toml::Value::Array(items) => {
                ensure_depth(depth)?;
                array_items = array_items.saturating_add(items.len());
                if array_items > MAX_PROJECT_MANIFEST_ARRAY_ITEMS {
                    return Err(ProjectManifestSummaryError::TooManyTomlArrayItems {
                        max: MAX_PROJECT_MANIFEST_ARRAY_ITEMS,
                        found: array_items,
                    });
                }
                let child_depth = depth.saturating_add(1);
                pending.extend(items.iter().map(|item| (item, child_depth)));
            }
            toml::Value::Table(table) => {
                ensure_depth(depth)?;
                table_entries = table_entries.saturating_add(table.len());
                if table_entries > MAX_PROJECT_MANIFEST_TABLE_ENTRIES {
                    return Err(ProjectManifestSummaryError::TooManyTomlTableEntries {
                        max: MAX_PROJECT_MANIFEST_TABLE_ENTRIES,
                        found: table_entries,
                    });
                }
                let child_depth = depth.saturating_add(1);
                pending.extend(table.values().map(|item| (item, child_depth)));
            }
            _ => {}
        }
    }

    Ok(())
}

/// 检查遍历到的 TOML 容器层级；根表为 0，每进入一层数组或子表加一。
/// 深度上限包含边界值，只有大于上限时才拒绝。
///
fn ensure_depth(depth: usize) -> Result<(), ProjectManifestSummaryError> {
    if depth > MAX_PROJECT_MANIFEST_NESTING_DEPTH {
        return Err(ProjectManifestSummaryError::TomlNestingTooDeep {
            max: MAX_PROJECT_MANIFEST_NESTING_DEPTH,
            found: depth,
        });
    }
    Ok(())
}
