use super::*;

#[test]
fn asset_import_error_preserves_registry_error_source() {
    let source_error = AssetImporterRegistryError::DuplicateMatcher {
        matcher: "ext:dup".to_string(),
        priority: 7,
    };

    let import_error = AssetImportError::from(source_error);

    match import_error {
        AssetImportError::Registry(AssetImporterRegistryError::DuplicateMatcher {
            matcher,
            priority,
        }) => {
            assert_eq!(matcher, "ext:dup");
            assert_eq!(priority, 7);
        }
        other => panic!("registry error source should remain typed, got {other:?}"),
    }
}
