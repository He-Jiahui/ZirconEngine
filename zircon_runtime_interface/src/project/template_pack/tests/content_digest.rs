use super::ProjectTemplateContentDigest;

#[test]
fn digest_is_independent_of_embedded_declaration_order() {
    let first = ProjectTemplateContentDigest::from_entries([
        ("b.txt", b"second".as_slice()),
        ("a.txt", b"first".as_slice()),
    ]);
    let second = ProjectTemplateContentDigest::from_entries([
        ("a.txt", b"first".as_slice()),
        ("b.txt", b"second".as_slice()),
    ]);

    assert_eq!(first, second);
}

#[test]
fn digest_changes_when_a_path_or_payload_changes() {
    let baseline = ProjectTemplateContentDigest::from_entries([("a.txt", b"first".as_slice())]);
    let changed_path = ProjectTemplateContentDigest::from_entries([("b.txt", b"first".as_slice())]);
    let changed_payload =
        ProjectTemplateContentDigest::from_entries([("a.txt", b"second".as_slice())]);

    assert_ne!(baseline, changed_path);
    assert_ne!(baseline, changed_payload);
}

#[test]
fn digest_serializes_as_lowercase_hex_and_round_trips() {
    let digest = ProjectTemplateContentDigest::from_entries([("a.txt", b"first".as_slice())]);
    let encoded = serde_json::to_string(&digest).expect("serialize template digest");
    let decoded: ProjectTemplateContentDigest =
        serde_json::from_str(&encoded).expect("deserialize template digest");

    assert_eq!(decoded, digest);
    assert_eq!(encoded.len(), 66);
    assert!(encoded[1..encoded.len() - 1]
        .bytes()
        .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte)));
}
