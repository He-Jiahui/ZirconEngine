use super::super::ProjectManifestDigest;

#[test]
fn project_manifest_digest_matches_blake3_content_identity() {
    let source = b"name = \"Digest Test\"\nformat_version = 3\nproject_guid = \"4cefc027-9a71-4c72-98e7-b327209c024c\"\n";

    let digest = ProjectManifestDigest::from_bytes(source);
    let expected = blake3::hash(source).to_hex().to_string();

    assert_eq!(digest.to_hex(), expected);
    assert_eq!(digest.to_string(), expected);
    assert_eq!(ProjectManifestDigest::parse(expected), Ok(digest));
    assert_eq!(
        serde_json::to_string(&digest).unwrap(),
        format!("\"{}\"", digest)
    );
}

#[test]
fn project_manifest_digest_changes_when_manifest_bytes_change() {
    let original = ProjectManifestDigest::from_bytes(b"name = \"Original\"\n");
    let replacement = ProjectManifestDigest::from_bytes(b"name = \"Replacement\"\n");

    assert_ne!(original, replacement);
}

#[test]
fn project_manifest_digest_rejects_noncanonical_wire_text() {
    assert!(ProjectManifestDigest::parse("not-a-digest").is_err());
    assert!(ProjectManifestDigest::parse("A".repeat(64)).is_err());
}
