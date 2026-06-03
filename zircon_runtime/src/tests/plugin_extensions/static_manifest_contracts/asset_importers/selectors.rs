use std::path::Path;

use super::super::{assert_unique_string_array_entries, non_empty_string_array_values};

pub(super) fn assert_asset_importer_source_selectors(
    importer: &toml::Table,
    relative_path: &Path,
    importer_context: &str,
) {
    let has_source_extensions = importer.get("source_extensions").is_some();
    let has_full_suffixes = importer.get("full_suffixes").is_some();
    assert!(
        has_source_extensions || has_full_suffixes,
        "plugin manifest {relative_path:?} {importer_context} should declare source_extensions or full_suffixes"
    );

    if has_source_extensions {
        assert_unique_string_array_entries(
            importer,
            relative_path,
            importer_context,
            "source_extensions",
        );
        for extension in non_empty_string_array_values(
            importer,
            relative_path,
            importer_context,
            "source_extensions",
        ) {
            let normalized = extension
                .trim()
                .trim_start_matches('.')
                .to_ascii_lowercase();
            assert_eq!(
                extension, normalized,
                "plugin manifest {relative_path:?} {importer_context} source extension `{extension}` should be lowercase without a leading dot"
            );
        }
    }

    if has_full_suffixes {
        assert_unique_string_array_entries(
            importer,
            relative_path,
            importer_context,
            "full_suffixes",
        );
        for suffix in non_empty_string_array_values(
            importer,
            relative_path,
            importer_context,
            "full_suffixes",
        ) {
            let normalized = if suffix.trim().starts_with('.') {
                suffix.trim().to_ascii_lowercase()
            } else {
                format!(".{}", suffix.trim().to_ascii_lowercase())
            };
            assert_eq!(
                suffix, normalized,
                "plugin manifest {relative_path:?} {importer_context} full suffix `{suffix}` should be lowercase and include the leading dot"
            );
            assert!(
                suffix.len() > 1,
                "plugin manifest {relative_path:?} {importer_context} full suffix `{suffix}` should include a suffix after the dot"
            );
        }
    }
}
