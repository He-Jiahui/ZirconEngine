use super::{parse_product_receipt_build_options, parse_product_receipt_draft_batch_issue_options};

#[test]
fn owned_batch_issue_options_preserve_values() {
    let options = parse_product_receipt_draft_batch_issue_options(strings(&[
        "--draft-batch",
        "drafts.json",
        "--expected-draft-sha256",
        "ABCDEF",
        "--private-key",
        "key.pk8",
        "--trust-registry",
        "trust.json",
        "--signer-id",
        "build-worker-01",
        "--created-utc",
        "2026-08-29T00:00:00Z",
        "--output",
        "receipt.json",
    ]))
    .unwrap();

    assert_eq!(options.draft_batch.to_str(), Some("drafts.json"));
    assert_eq!(options.expected_draft_sha256, "ABCDEF");
    assert_eq!(options.private_key.to_str(), Some("key.pk8"));
    assert_eq!(options.trust_registry.to_str(), Some("trust.json"));
    assert_eq!(options.signer_id, "build-worker-01");
    assert_eq!(options.created_utc, "2026-08-29T00:00:00Z");
    assert_eq!(options.output.to_str(), Some("receipt.json"));
}

#[test]
fn owned_options_reject_duplicates_and_missing_values() {
    assert!(parse_product_receipt_build_options(strings(&[
        "--request",
        "first.json",
        "--request",
        "second.json",
        "--output",
        "draft.json",
    ]))
    .is_err());
    assert!(parse_product_receipt_build_options(strings(&["--request"])).is_err());
}

fn strings(values: &[&str]) -> Vec<String> {
    values.iter().map(|value| (*value).to_string()).collect()
}
