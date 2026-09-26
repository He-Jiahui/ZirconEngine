use super::{RelPath, RelPathError};

/// 为清单资产根和持久化引用提供统一的词法相对路径；实际落盘仍须由调用方验证根目录约束。
pub(super) fn parse(value: &str) -> Result<RelPath, RelPathError> {
    if value.is_empty() {
        return Err(RelPathError::Empty);
    }

    let portable = value.replace('\\', "/");
    if portable.starts_with('/') || has_platform_prefix(&portable) {
        return Err(RelPathError::AbsoluteOrPrefixed {
            path: value.to_string(),
        });
    }

    let mut normalized = Vec::new();
    for component in portable.split('/') {
        if component.is_empty() {
            continue;
        }
        if matches!(component, "." | "..") {
            return Err(RelPathError::DotComponent {
                path: value.to_string(),
            });
        }
        normalized.push(component);
    }
    if normalized.is_empty() {
        return Err(RelPathError::Empty);
    }
    Ok(RelPath(normalized.join("/")))
}

// BUG: [CR-PROJECT-0001] 仅检查首段会放过 safe/C:/outside；Windows PathBuf 将其转为
// C:outside，join_to 因而逸出项目根。清单资产根随后会交给 create_dir_all。
fn has_platform_prefix(value: &str) -> bool {
    let first = value.split('/').next().unwrap_or_default();
    let bytes = first.as_bytes();
    bytes.len() >= 2 && bytes[0].is_ascii_alphabetic() && bytes[1] == b':'
}
