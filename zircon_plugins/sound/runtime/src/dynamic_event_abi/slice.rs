use zircon_runtime_interface::ZrByteSlice;

pub(crate) fn borrowed_slice(bytes: &[u8]) -> ZrByteSlice {
    if bytes.is_empty() {
        ZrByteSlice::empty()
    } else {
        ZrByteSlice {
            data: bytes.as_ptr(),
            len: bytes.len(),
        }
    }
}
