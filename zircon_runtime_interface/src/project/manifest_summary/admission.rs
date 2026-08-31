use super::{
    ProjectManifestSummaryError, MAX_PROJECT_MANIFEST_ARRAY_ITEMS,
    MAX_PROJECT_MANIFEST_NESTING_DEPTH, MAX_PROJECT_MANIFEST_TABLE_ENTRIES,
};

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
                array_items = array_items.checked_add(items.len()).unwrap_or(usize::MAX);
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
                table_entries = table_entries.checked_add(table.len()).unwrap_or(usize::MAX);
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

fn ensure_depth(depth: usize) -> Result<(), ProjectManifestSummaryError> {
    if depth > MAX_PROJECT_MANIFEST_NESTING_DEPTH {
        return Err(ProjectManifestSummaryError::TomlNestingTooDeep {
            max: MAX_PROJECT_MANIFEST_NESTING_DEPTH,
            found: depth,
        });
    }
    Ok(())
}
