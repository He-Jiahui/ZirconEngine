use super::{RelPath, RelPathError};

/// 为清单资产根和持久化引用提供统一的词法相对路径；实际落盘仍须由调用方验证根目录约束。
pub(super) fn parse(value: &str) -> Result<RelPath, RelPathError> {
    if value.is_empty() {
        return Err(RelPathError::Empty);
    }

    let portable = value.replace('\\', "/");
    if portable.starts_with('/') || portable.split('/').any(is_windows_unsafe_component) {
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

// Windows treats a colon in any path component as a drive/alternate-stream marker, and
// device names remain reserved even when followed by an extension (for example, CON.txt).
fn is_windows_unsafe_component(component: &str) -> bool {
    if component.contains(':') {
        return true;
    }

    // Windows strips trailing spaces and periods before resolving a path component.
    let normalized = component.trim_end_matches([' ', '.']);
    let device_stem = normalized
        .split('.')
        .next()
        .unwrap_or_default()
        .trim_end_matches([' ', '.']);
    let device_stem = device_stem.to_ascii_uppercase();
    matches!(device_stem.as_str(), "CON" | "PRN" | "AUX" | "NUL" | "CONIN$" | "CONOUT$")
        || ["COM", "LPT"].iter().any(|prefix| {
            device_stem.strip_prefix(prefix).is_some_and(|suffix| {
                matches!(suffix, "1" | "2" | "3" | "4" | "5" | "6" | "7" | "8" | "9" | "¹" | "²" | "³")
            })
        })
}
