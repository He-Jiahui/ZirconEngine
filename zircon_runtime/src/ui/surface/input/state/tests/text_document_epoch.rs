use super::*;

#[test]
fn exhausted_epoch_never_aliases_an_earlier_document_source() {
    let owner = UiNodeId::new(7);
    let mut epochs = UiTextDocumentEpochs::default();
    epochs.revisions.insert(owner, Some(u64::MAX));

    assert_eq!(epochs.advance(owner), None);
    assert_eq!(epochs.current(owner), None);
    assert_eq!(epochs.advance(owner), None);
}
