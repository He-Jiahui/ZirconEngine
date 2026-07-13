use super::{RelPath, RelPathError};

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

fn has_platform_prefix(value: &str) -> bool {
    let first = value.split('/').next().unwrap_or_default();
    let bytes = first.as_bytes();
    bytes.len() >= 2 && bytes[0].is_ascii_alphabetic() && bytes[1] == b':'
}
