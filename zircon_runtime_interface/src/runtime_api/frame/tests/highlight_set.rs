use super::{ZrRuntimeHighlightRenderAttributesV1, ZrRuntimeHighlightSetV1};
use crate::handles::ZrRuntimeViewportHandle;

#[test]
fn validates_a_borrowed_entity_slice_without_owning_it() {
    let entities = [7, 2, 7];
    let request = ZrRuntimeHighlightSetV1::new(
        ZrRuntimeViewportHandle::new(5),
        3,
        &entities,
        ZrRuntimeHighlightRenderAttributesV1::outlined([0.4, 0.6, 0.8, 1.0]),
    );

    assert!(unsafe { request.validate() });
    assert_eq!(
        unsafe { request.entities.as_slice() },
        Some(entities.as_slice())
    );
}

#[test]
fn rejects_misaligned_or_oversized_borrowed_slices() {
    let misaligned = super::ZrRuntimeEntityIdSliceV1 {
        data: 1_usize as *const u64,
        len: 1,
    };
    let oversized = super::ZrRuntimeEntityIdSliceV1 {
        data: core::ptr::NonNull::<u64>::dangling().as_ptr(),
        len: (isize::MAX as usize / core::mem::size_of::<u64>()) + 1,
    };

    assert!(unsafe { misaligned.as_slice() }.is_none());
    assert!(unsafe { oversized.as_slice() }.is_none());
}
