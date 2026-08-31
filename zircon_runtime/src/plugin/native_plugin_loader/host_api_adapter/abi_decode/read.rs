use zircon_runtime_interface::{
    ZrByteSlice, ZR_RUNTIME_NATIVE_STRING_LIST_MAX_ITEMS_V1,
    ZR_RUNTIME_NATIVE_STRING_MAX_ENCODED_BYTES_V1,
};

use super::{AbiDecodeError, AbiDecodeResult};

pub(in super::super) unsafe fn read_byte_slices(
    values: *const ZrByteSlice,
    count: usize,
) -> AbiDecodeResult<Vec<String>> {
    if count == 0 {
        return Ok(Vec::new());
    }
    if values.is_null()
        || count > ZR_RUNTIME_NATIVE_STRING_LIST_MAX_ITEMS_V1
        || count > isize::MAX as usize / std::mem::size_of::<ZrByteSlice>()
        || values.align_offset(std::mem::align_of::<ZrByteSlice>()) != 0
    {
        return Err(AbiDecodeError::InvalidV4StringListPointer {
            field: "string list",
            count,
        });
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
    unsafe { read_utf8_with(slice, str::to_string) }
}

pub(in super::super) unsafe fn read_utf8_with<T>(
    slice: ZrByteSlice,
    visitor: impl FnOnce(&str) -> T,
) -> AbiDecodeResult<T> {
    let bytes = unsafe { slice.checked_slice(ZR_RUNTIME_NATIVE_STRING_MAX_ENCODED_BYTES_V1) }
        .map_err(|_| AbiDecodeError::InvalidV4StringListPointer {
            field: "string value",
            count: slice.len,
        })?;
    let value =
        std::str::from_utf8(bytes).map_err(|source| AbiDecodeError::InvalidUtf8 { source })?;
    Ok(visitor(value))
}

#[cfg(test)]
mod tests {
    use std::mem::MaybeUninit;

    use super::*;

    #[test]
    fn byte_slice_list_rejects_misaligned_storage_before_dereference() {
        let storage = [MaybeUninit::<ZrByteSlice>::uninit(); 2];
        let misaligned = unsafe { storage.as_ptr().cast::<u8>().add(1).cast::<ZrByteSlice>() };

        let error = unsafe { read_byte_slices(misaligned, 1) }
            .expect_err("misaligned foreign list storage must be rejected");

        assert!(matches!(
            error,
            AbiDecodeError::InvalidV4StringListPointer {
                field: "string list",
                count: 1
            }
        ));
    }

    #[test]
    fn borrowed_utf8_mapping_preserves_exact_projection() {
        let projected = unsafe {
            read_utf8_with(ZrByteSlice::from_static(b"weather.velocity"), |stable_id| {
                format!("read:component:{stable_id}")
            })
        }
        .unwrap();

        assert_eq!(projected, "read:component:weather.velocity");
    }
}
