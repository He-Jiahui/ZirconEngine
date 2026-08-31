use super::super::ReflectError;

pub const MAX_REFLECT_TYPE_PATH_BYTES: usize = 512;
pub const MAX_REFLECT_SHORT_TYPE_PATH_BYTES: usize = 128;
pub const MAX_REFLECT_MODULE_PATH_BYTES: usize = 384;
pub const MAX_REFLECT_PLUGIN_ID_BYTES: usize = 128;

pub(in crate::reflect) fn validate_type_path(type_path: &str) -> Result<(), ReflectError> {
    validate_length(
        type_path,
        "type path",
        MAX_REFLECT_TYPE_PATH_BYTES,
        type_path,
    )?;
    validate_canonical_text(type_path, "type path", type_path)?;

    let has_rust_separator = type_path.contains("::");
    let has_vm_separator = type_path.contains('.');
    if has_rust_separator && has_vm_separator {
        return Err(invalid_type_path(
            type_path,
            "type path must not mix `::` and `.` separators",
        ));
    }

    let valid_segments = if has_rust_separator {
        type_path.split("::").all(is_identifier)
    } else if has_vm_separator {
        validate_vm_type_path(type_path)
    } else {
        is_identifier(type_path)
    };
    if !valid_segments {
        return Err(invalid_type_path(
            type_path,
            "type path segments do not match the selected Rust or VM grammar",
        ));
    }

    Ok(())
}

pub(super) fn validate_short_type_path(
    type_path: &str,
    short_type_path: &str,
) -> Result<(), ReflectError> {
    validate_length(
        short_type_path,
        "short type path",
        MAX_REFLECT_SHORT_TYPE_PATH_BYTES,
        type_path,
    )?;
    validate_canonical_text(short_type_path, "short type path", type_path)?;
    if !is_identifier(short_type_path) {
        return Err(invalid_type_path(
            type_path,
            "short type path must be one ASCII identifier",
        ));
    }

    let terminal = terminal_segment(type_path);
    if short_type_path != terminal {
        return Err(invalid_type_path(
            type_path,
            format!("short type path must match terminal segment `{terminal}`"),
        ));
    }

    Ok(())
}

pub(super) fn validate_module_path(type_path: &str, module_path: &str) -> Result<(), ReflectError> {
    validate_length(
        module_path,
        "module path",
        MAX_REFLECT_MODULE_PATH_BYTES,
        type_path,
    )?;
    validate_canonical_text(module_path, "module path", type_path)?;

    let expected = module_prefix(type_path).ok_or_else(|| {
        invalid_type_path(
            type_path,
            "module path cannot be attached to an unqualified type path",
        )
    })?;
    if module_path != expected {
        return Err(invalid_type_path(
            type_path,
            format!("module path must match full type path prefix `{expected}`"),
        ));
    }
    Ok(())
}

pub(super) fn validate_plugin_id(type_path: &str, plugin_id: &str) -> Result<(), ReflectError> {
    validate_length(
        plugin_id,
        "plugin id",
        MAX_REFLECT_PLUGIN_ID_BYTES,
        type_path,
    )?;
    validate_canonical_text(plugin_id, "plugin id", type_path)?;

    let mut bytes = plugin_id.bytes();
    let Some(first) = bytes.next() else {
        return Err(invalid_type_path(type_path, "plugin id must not be empty"));
    };
    if !first.is_ascii_alphanumeric()
        || first.is_ascii_uppercase()
        || bytes.any(|byte| {
            byte.is_ascii_uppercase()
                || (!byte.is_ascii_alphanumeric() && !matches!(byte, b'_' | b'-' | b'.'))
        })
    {
        return Err(invalid_type_path(
            type_path,
            "plugin id must be a canonical lowercase ASCII key",
        ));
    }
    Ok(())
}

fn validate_length(
    value: &str,
    field: &str,
    max_bytes: usize,
    type_path: &str,
) -> Result<(), ReflectError> {
    if value.len() > max_bytes {
        return Err(invalid_type_path(
            type_path,
            format!("{field} exceeds {max_bytes} bytes"),
        ));
    }
    Ok(())
}

fn validate_canonical_text(value: &str, field: &str, type_path: &str) -> Result<(), ReflectError> {
    if value.is_empty() {
        return Err(invalid_type_path(
            type_path,
            format!("{field} must not be empty"),
        ));
    }
    if value.trim() != value {
        return Err(invalid_type_path(
            type_path,
            format!("{field} must not have surrounding whitespace"),
        ));
    }
    if !value.is_ascii() {
        return Err(invalid_type_path(
            type_path,
            format!("{field} must be ASCII"),
        ));
    }
    Ok(())
}

fn is_identifier(segment: &str) -> bool {
    let mut bytes = segment.bytes();
    let Some(first) = bytes.next() else {
        return false;
    };
    (first.is_ascii_alphabetic() || first == b'_')
        && bytes.all(|byte| byte.is_ascii_alphanumeric() || byte == b'_')
}

fn validate_vm_type_path(type_path: &str) -> bool {
    let mut segments = type_path.split('.').peekable();
    while let Some(segment) = segments.next() {
        if segments.peek().is_none() {
            return is_identifier(segment);
        }
        if !is_vm_namespace_segment(segment) {
            return false;
        }
    }
    false
}

fn is_vm_namespace_segment(segment: &str) -> bool {
    let mut bytes = segment.bytes();
    let Some(first) = bytes.next() else {
        return false;
    };
    (first.is_ascii_alphanumeric() || first == b'_')
        && bytes.all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'_' | b'-'))
}

fn terminal_segment(type_path: &str) -> &str {
    if type_path.contains("::") {
        type_path.rsplit("::").next().unwrap_or(type_path)
    } else {
        type_path.rsplit('.').next().unwrap_or(type_path)
    }
}

fn module_prefix(type_path: &str) -> Option<&str> {
    if type_path.contains("::") {
        type_path.rsplit_once("::").map(|(prefix, _)| prefix)
    } else {
        type_path.rsplit_once('.').map(|(prefix, _)| prefix)
    }
}

fn invalid_type_path(type_path: &str, reason: impl Into<String>) -> ReflectError {
    ReflectError::InvalidTypePath {
        type_path: type_path.to_string(),
        reason: reason.into(),
    }
}
