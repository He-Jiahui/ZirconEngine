use zircon_runtime_interface::ZrByteSlice;

pub(in crate::entry::runtime_entry_app) fn byte_slice(value: &str) -> ZrByteSlice {
    ZrByteSlice {
        data: value.as_bytes().as_ptr(),
        len: value.len(),
    }
}

pub(in crate::entry::runtime_entry_app) fn usize_to_u32(value: usize) -> u32 {
    u32::try_from(value).unwrap_or(u32::MAX - 1)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn byte_and_usize_helpers_match_abi_bounds() {
        let payload = byte_slice("abc");
        assert_eq!(payload.len, 3);
        assert!(!payload.data.is_null());
        assert_eq!(usize_to_u32(7), 7);
        assert_eq!(usize_to_u32(usize::MAX), u32::MAX - 1);
    }
}
