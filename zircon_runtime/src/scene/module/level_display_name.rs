use crate::core::resource::ResourceLocator;

pub(super) fn display_name_for_level(uri: &ResourceLocator) -> Option<String> {
    let source = uri.label().unwrap_or(uri.path());
    source.rsplit('/').next().map(ToString::to_string)
}

#[cfg(test)]
#[path = "tests/level_display_name.rs"]
mod tests;
