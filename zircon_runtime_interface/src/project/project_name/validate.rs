use std::path::{Component, Path};

use super::ProjectNameError;

pub fn validate_project_name(value: &str) -> Result<(), ProjectNameError> {
    if value.is_empty() || value.trim().is_empty() {
        return Err(ProjectNameError::Empty);
    }
    if value != value.trim() {
        return Err(ProjectNameError::SurroundingWhitespace {
            value: value.to_string(),
        });
    }
    if matches!(value, "." | "..")
        || value.contains('/')
        || value.contains('\\')
        || Path::new(value).is_absolute()
        || !matches!(
            Path::new(value).components().collect::<Vec<_>>().as_slice(),
            [Component::Normal(_)]
        )
    {
        return Err(ProjectNameError::NotSingleComponent {
            value: value.to_string(),
        });
    }
    if value.ends_with('.') || value.ends_with(' ') {
        return Err(ProjectNameError::WindowsTrailingAlias {
            value: value.to_string(),
        });
    }
    if value
        .chars()
        .any(|character| character.is_control() || "<>:\"|?*".contains(character))
    {
        return Err(ProjectNameError::ForbiddenCharacter {
            value: value.to_string(),
        });
    }
    let basename = value
        .split('.')
        .next()
        .unwrap_or(value)
        .to_ascii_uppercase();
    if is_windows_reserved(&basename) {
        return Err(ProjectNameError::WindowsReserved {
            value: value.to_string(),
        });
    }
    Ok(())
}

fn is_windows_reserved(value: &str) -> bool {
    matches!(value, "CON" | "PRN" | "AUX" | "NUL")
        || value
            .strip_prefix("COM")
            .or_else(|| value.strip_prefix("LPT"))
            .is_some_and(|suffix| {
                matches!(suffix, "1" | "2" | "3" | "4" | "5" | "6" | "7" | "8" | "9")
            })
}
