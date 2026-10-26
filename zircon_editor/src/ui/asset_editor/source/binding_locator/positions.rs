use std::ops::Range;

use super::{UiAssetBindingSourceError, UiAssetSourcePosition, UiAssetSourceRange};

pub(super) fn source_range(
    source: &str,
    range: Range<usize>,
) -> Result<UiAssetSourceRange, UiAssetBindingSourceError> {
    if range.start > range.end || source.get(range.clone()).is_none() {
        return Err(UiAssetBindingSourceError::InvalidRange);
    }
    Ok(UiAssetSourceRange {
        start: source_position(source, range.start)?,
        end: source_position(source, range.end)?,
    })
}

fn source_position(
    source: &str,
    byte_offset: usize,
) -> Result<UiAssetSourcePosition, UiAssetBindingSourceError> {
    let prefix = source
        .get(..byte_offset)
        .ok_or(UiAssetBindingSourceError::InvalidRange)?;
    let line_start = prefix.rfind('\n').map_or(0, |index| index + 1);
    // A CRLF is one line ending. There is no LSP position between CR and LF.
    if prefix.ends_with('\r') && source.as_bytes().get(byte_offset) == Some(&b'\n') {
        return Err(UiAssetBindingSourceError::InvalidRange);
    }
    Ok(UiAssetSourcePosition {
        byte_offset,
        line: prefix.bytes().filter(|byte| *byte == b'\n').count() + 1,
        utf16_column: source[line_start..byte_offset].encode_utf16().count(),
    })
}

#[cfg(test)]
#[path = "tests/positions.rs"]
mod tests;
