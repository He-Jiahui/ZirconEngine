use super::UiAssetSourceBuffer;

#[test]
fn revision_changes_only_when_source_text_changes() {
    let mut buffer = UiAssetSourceBuffer::new("[nodes.root]");

    buffer.replace("[nodes.root]");
    assert_eq!(buffer.revision(), 0);

    buffer.replace("[nodes.root]\nkind = \"native\"");
    assert_eq!(buffer.revision(), 1);
}
