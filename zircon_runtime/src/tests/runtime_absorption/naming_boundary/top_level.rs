use std::collections::BTreeSet;
use std::path::Path;

use super::classifiers::{
    allowed_server_context, classify_editor_reference, classify_legacy_reference,
    classify_server_reference,
};
use super::lexical_scan::{
    collect_naming_references, collect_server_references, rust_source_files, NamingReference,
};

#[test]
fn runtime_editor_and_legacy_naming_is_classified_by_owner() {
    let manifest_root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let source_root = manifest_root.join("src");
    let files = rust_source_files(&source_root);

    assert_no_unclassified_naming(
        "editor",
        &collect_naming_references(manifest_root, &files, "editor"),
        classify_editor_reference,
    );
    assert_no_unclassified_naming(
        "legacy",
        &collect_naming_references(manifest_root, &files, "legacy"),
        classify_legacy_reference,
    );
}

#[test]
fn runtime_non_network_server_naming_is_classified_by_owner() {
    let manifest_root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let source_root = manifest_root.join("src");
    let files = rust_source_files(&source_root);
    let references = collect_server_references(manifest_root, &files);

    let mut classifications = BTreeSet::new();
    let unclassified = references
        .iter()
        .filter(|reference| {
            if allowed_server_context(&reference.path, &reference.snippet) {
                return false;
            }
            match classify_server_reference(&reference.path, &reference.snippet) {
                Some(classification) => {
                    classifications.insert(classification);
                    false
                }
                None => true,
            }
        })
        .take(20)
        .map(|reference| {
            format!(
                "{}:{}: {}",
                reference.path, reference.line, reference.snippet
            )
        })
        .collect::<Vec<_>>();

    assert!(
        unclassified.is_empty(),
        "runtime non-network server naming contains unclassified owner references:\n{}",
        unclassified.join("\n")
    );
    assert!(
        classifications
            .iter()
            .all(|classification| *classification != "unclassified-non-network-server"),
        "runtime non-network server naming guard should never classify unknown owner debt"
    );
}

fn assert_no_unclassified_naming(
    term: &str,
    references: &[NamingReference],
    classifier: fn(&str) -> Option<&'static str>,
) {
    let mut classifications = BTreeSet::new();
    let unclassified = references
        .iter()
        .filter(|reference| match classifier(&reference.path) {
            Some(classification) => {
                classifications.insert(classification);
                false
            }
            None => true,
        })
        .take(20)
        .map(|reference| {
            format!(
                "{}:{}: {}",
                reference.path, reference.line, reference.snippet
            )
        })
        .collect::<Vec<_>>();

    assert!(
        !classifications.is_empty(),
        "runtime {term} naming guard should classify at least one owner bucket"
    );
    assert!(
        unclassified.is_empty(),
        "runtime {term} naming contains unclassified owner references:\n{}",
        unclassified.join("\n")
    );
}
