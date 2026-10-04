use super::*;
#[test]
fn credential_capacity_is_the_actual_byte_limit_and_revoke_does_not_grow_it() {
    let mut saved = SavedSession {
        issuer: "issuer".into(),
        subject: "alice".into(),
        nonce: "nonce".into(),
        refresh_token: String::new(),
        revoke_only: false,
    };
    let overhead = encode(&saved).unwrap().len();
    saved.refresh_token = "x".repeat(CREDENTIAL_BLOB_BYTES - overhead);
    assert_eq!(encode(&saved).unwrap().len(), CREDENTIAL_BLOB_BYTES);
    saved.revoke_only = true;
    assert!(encode(&saved).unwrap().len() < CREDENTIAL_BLOB_BYTES);
    saved.refresh_token.push_str("xx");
    assert!(matches!(encode(&saved), Err(AccountError::CredentialStore)));
}
#[test]
fn existing_utf16_and_new_utf8_credentials_decode_without_exposing_tokens() {
    let legacy =
        r#"{"issuer":"issuer","subject":"alice","nonce":"nonce","refresh_token":"fixture"}"#;
    let bytes: Vec<u8> = legacy.encode_utf16().flat_map(u16::to_le_bytes).collect();
    let mut saved = decode(&bytes).unwrap();
    assert!(!saved.revoke_only);
    saved.revoke_only = true;
    assert!(decode(&encode(&saved).unwrap()).unwrap().revoke_only);
}
