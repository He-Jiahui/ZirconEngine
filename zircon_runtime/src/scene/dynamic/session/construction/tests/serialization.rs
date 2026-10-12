use super::*;

#[test]
fn runtime_session_archive_text_input_rejects_before_json_decode_when_oversized() {
    let error = ensure_archive_input_limit(2, 1).unwrap_err();

    assert!(matches!(
        error,
        RuntimeSessionArchiveError::ArtifactTooLarge {
            estimated_bytes: 2,
            limit_bytes: 1,
        }
    ));
}
