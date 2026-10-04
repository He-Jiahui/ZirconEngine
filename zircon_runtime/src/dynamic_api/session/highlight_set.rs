use zircon_runtime_interface::ZrRuntimeHighlightSetV1;

use crate::core::framework::render::{HighlightRenderAttributes, HighlightSet};

use super::RuntimeDynamicSession;

impl RuntimeDynamicSession {
    pub(super) fn submit_highlight_set(&mut self, request: ZrRuntimeHighlightSetV1) {
        // SAFETY: FFI 闭包用仍存活的 Vec 重建并验证切片；本次同步调用期间该 Vec 未被修改。
        let entities = unsafe { request.entities.as_slice() }
            .expect("highlight set payload is validated at the FFI boundary");
        self.level.submit_highlight_set(
            request.viewport.raw(),
            request.generation,
            HighlightSet::new(
                entities.iter().copied(),
                HighlightRenderAttributes {
                    outline_enabled: request.attributes.outline_enabled != 0,
                    tint_rgba: request.attributes.tint_rgba,
                },
            ),
        );
    }
}
