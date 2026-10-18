use super::Catalog;
use serde::Deserialize;

#[derive(Deserialize)]
struct EntryWithoutDefault {
    value: u32,
}

#[test]
fn generic_catalog_does_not_require_default_entries() {
    let catalog: Catalog<EntryWithoutDefault> = serde_json::from_str(
        r#"{"schema_version":1,"source_commit":"fixture","entries":[{"value":7}]}"#,
    )
    .expect("catalog defaults must not impose T: Default");

    assert_eq!(catalog.entries[0].value, 7);
    assert!(catalog.files.is_empty());
    assert!(catalog.generators.is_empty());
}
