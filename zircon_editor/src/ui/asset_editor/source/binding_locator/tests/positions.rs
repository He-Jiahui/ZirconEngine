use super::*;

#[test]
fn positions_preserve_crlf_cjk_and_surrogate_pairs() {
    let source = "首行\r\n中文😀='值'\r\n";
    let byte = source.find('=').unwrap();
    let range = source_range(source, byte..byte + 1).unwrap();
    assert_eq!(range.start.line, 2);
    assert_eq!(range.start.utf16_column, 4);
    assert_eq!(range.end.utf16_column, 5);
    assert_eq!(range.start.byte_offset, byte);
    assert_eq!(
        source_range(source, 1..2),
        Err(UiAssetBindingSourceError::InvalidRange)
    );
    let lf = source.find('\n').unwrap();
    assert_eq!(
        source_range(source, lf..lf),
        Err(UiAssetBindingSourceError::InvalidRange)
    );
}
