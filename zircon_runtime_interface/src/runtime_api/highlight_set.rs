use crate::handles::ZrRuntimeViewportHandle;
use crate::version::ZIRCON_RUNTIME_ABI_VERSION_V1;

#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ZrRuntimeEntityIdSliceV1 {
    pub data: *const u64,
    pub len: usize,
}

impl ZrRuntimeEntityIdSliceV1 {
    pub const fn empty() -> Self {
        Self {
            data: core::ptr::null(),
            len: 0,
        }
    }

    pub fn from_slice(values: &[u64]) -> Self {
        Self {
            data: values.as_ptr(),
            len: values.len(),
        }
    }

    /// # Safety
    ///
    /// The caller must ensure `data` points to `len` readable `u64` values for the returned
    /// slice lifetime.
    pub unsafe fn as_slice(&self) -> Option<&[u64]> {
        if self.len == 0 {
            return Some(&[]);
        }
        let max_elements = isize::MAX as usize / core::mem::size_of::<u64>();
        if self.data.is_null()
            || (self.data as usize) % core::mem::align_of::<u64>() != 0
            || self.len > max_elements
        {
            return None;
        }
        Some(unsafe { core::slice::from_raw_parts(self.data, self.len) })
    }
}

#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ZrRuntimeHighlightRenderAttributesV1 {
    pub outline_enabled: u32,
    pub tint_rgba: [f32; 4],
}

impl ZrRuntimeHighlightRenderAttributesV1 {
    pub const OUTLINE_DISABLED: u32 = 0;
    pub const OUTLINE_ENABLED: u32 = 1;

    pub const fn outlined(tint_rgba: [f32; 4]) -> Self {
        Self {
            outline_enabled: Self::OUTLINE_ENABLED,
            tint_rgba,
        }
    }

    pub fn is_valid(self) -> bool {
        matches!(
            self.outline_enabled,
            Self::OUTLINE_DISABLED | Self::OUTLINE_ENABLED
        ) && self.tint_rgba.iter().all(|value| value.is_finite())
    }
}

#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ZrRuntimeHighlightSetV1 {
    pub abi_version: u32,
    pub viewport: ZrRuntimeViewportHandle,
    pub generation: u64,
    pub entities: ZrRuntimeEntityIdSliceV1,
    pub attributes: ZrRuntimeHighlightRenderAttributesV1,
}

impl ZrRuntimeHighlightSetV1 {
    pub fn new(
        viewport: ZrRuntimeViewportHandle,
        generation: u64,
        entities: &[u64],
        attributes: ZrRuntimeHighlightRenderAttributesV1,
    ) -> Self {
        Self {
            abi_version: ZIRCON_RUNTIME_ABI_VERSION_V1,
            viewport,
            generation,
            entities: ZrRuntimeEntityIdSliceV1::from_slice(entities),
            attributes,
        }
    }

    /// # Safety
    ///
    /// The borrowed entity slice must remain readable for this validation.
    pub unsafe fn validate(self) -> bool {
        self.abi_version == ZIRCON_RUNTIME_ABI_VERSION_V1
            && self.viewport.is_valid()
            && self.attributes.is_valid()
            && unsafe { self.entities.as_slice() }.is_some()
    }
}

#[cfg(test)]
mod tests {
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
}
