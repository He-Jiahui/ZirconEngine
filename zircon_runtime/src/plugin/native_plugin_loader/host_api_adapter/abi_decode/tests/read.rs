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
