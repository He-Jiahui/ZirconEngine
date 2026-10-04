use super::*;

#[test]
fn shared_import_reasons_stay_inline_and_keep_reason_order() {
    assert_eq!(
        std::mem::size_of::<SharedImportReasons>(),
        std::mem::size_of::<std::sync::atomic::AtomicU8>()
    );

    let reasons = SharedImportReasons::default();
    reasons.add(EditorAssetImportReason::Manual);
    reasons.add(EditorAssetImportReason::Watch);
    reasons.add(EditorAssetImportReason::DigestMismatch);
    reasons.add(EditorAssetImportReason::Manual);

    assert_eq!(
        reasons.snapshot(),
        vec![
            EditorAssetImportReason::Watch,
            EditorAssetImportReason::DigestMismatch,
            EditorAssetImportReason::Manual,
        ]
    );
    assert_eq!(reasons.len(), 3);
}
