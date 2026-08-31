use serde::Serialize;

use super::write_pretty_json;

#[derive(Serialize)]
struct ReceiptFixture<'a> {
    schema_version: u32,
    receipt_kind: &'a str,
    artifact_names: &'a [&'a str],
}

#[test]
fn streaming_pretty_json_matches_the_legacy_buffered_bytes() {
    let fixture = ReceiptFixture {
        schema_version: 1,
        receipt_kind: "zircon_product_receipt",
        artifact_names: &["runtime-executable", "runtime-library", "runtime-symbols"],
    };
    let expected = serde_json::to_vec_pretty(&fixture).unwrap();
    let mut actual = Vec::new();

    write_pretty_json(&mut actual, &fixture).unwrap();

    assert_eq!(actual, expected);
}
