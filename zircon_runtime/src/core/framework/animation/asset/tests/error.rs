use super::AnimationAssetError;

#[test]
fn binary_kind_mismatch_survives_document_and_schema_fallback_errors() {
    let error = AnimationAssetError::CurrentAndV1PayloadDecode {
        kind: "sequence",
        current: Box::new(AnimationAssetError::DocumentAndStreamDecode {
            kind: "sequence",
            document: Box::new(AnimationAssetError::KindMismatch {
                expected: "sequence",
                actual: "graph",
            }),
            stream: Box::new(AnimationAssetError::InvalidMagic),
        }),
        v1: Box::new(AnimationAssetError::InvalidMagic),
    };

    assert_eq!(error.binary_kind_mismatch(), Some(("sequence", "graph")));
    assert_eq!(
        AnimationAssetError::InvalidMagic.binary_kind_mismatch(),
        None
    );
}
