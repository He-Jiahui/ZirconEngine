use zircon_runtime_interface::ZrByteSlice;

use super::{AbiDecodeError, AbiDecodeResult};

pub(in super::super) unsafe fn read_byte_slices(
    values: *const ZrByteSlice,
    count: usize,
) -> AbiDecodeResult<Vec<String>> {
    if values.is_null() || count == 0 {
        return Ok(Vec::new());
    }
    unsafe { std::slice::from_raw_parts(values, count) }
        .iter()
        .copied()
        .map(|slice| unsafe { read_utf8(slice) })
        .collect()
}

pub(in super::super) unsafe fn read_v4_byte_slices(
    field: &'static str,
    values: *const ZrByteSlice,
    count: usize,
) -> AbiDecodeResult<Vec<String>> {
    if count == 0 {
        return Ok(Vec::new());
    }
    if values.is_null() {
        return Err(AbiDecodeError::InvalidV4StringListPointer { field, count });
    }
    unsafe { read_byte_slices(values, count) }
}

pub(in super::super) unsafe fn read_utf8(slice: ZrByteSlice) -> AbiDecodeResult<String> {
    std::str::from_utf8(unsafe { slice.as_slice() })
        .map(str::to_string)
        .map_err(|source| AbiDecodeError::InvalidUtf8 { source })
}
