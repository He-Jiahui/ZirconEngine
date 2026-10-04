use super::*;

#[test]
fn asset_meta_validation_reports_typed_future_version_error() {
    let mut meta = AssetMetaDocument::new(
        AssetUuid::new(),
        AssetUri::parse("res://data/future.json").unwrap(),
        AssetKind::Data,
    );
    meta.format_version = ASSET_META_FORMAT_VERSION + 1;

    let error = meta
        .validate_current()
        .expect_err("future meta version should fail");

    assert_eq!(
        error,
        AssetMetaError::UnsupportedFutureFormatVersion {
            found: ASSET_META_FORMAT_VERSION + 1,
            supported: ASSET_META_FORMAT_VERSION,
        }
    );
}
