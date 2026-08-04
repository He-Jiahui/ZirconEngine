use zircon_runtime_interface::{
    ZrRuntimeOperationHandle, ZrRuntimeOperationResultV1, ZrRuntimeOperationStatusV2,
    ZrRuntimeOperationSubmitRequestV1, ZrRuntimeSessionHandle,
};

use super::{EditorRuntimeGateway, GatewayError};

#[derive(Debug, Default)]
pub struct DetachedEditorRuntimeGateway;

impl EditorRuntimeGateway for DetachedEditorRuntimeGateway {
    fn session_handle(&self) -> ZrRuntimeSessionHandle {
        ZrRuntimeSessionHandle::invalid()
    }

    fn submit_operation(
        &self,
        _request: ZrRuntimeOperationSubmitRequestV1,
    ) -> Result<ZrRuntimeOperationHandle, GatewayError> {
        Err(GatewayError::CapabilityMissing {
            capability: "runtime.operation.submit",
        })
    }

    fn poll_operation(
        &self,
        _handle: ZrRuntimeOperationHandle,
    ) -> Result<ZrRuntimeOperationStatusV2, GatewayError> {
        Err(GatewayError::CapabilityMissing {
            capability: "runtime.operation.poll",
        })
    }

    fn harvest_operation(
        &self,
        _handle: ZrRuntimeOperationHandle,
    ) -> Result<ZrRuntimeOperationResultV1, GatewayError> {
        Err(GatewayError::CapabilityMissing {
            capability: "runtime.operation.harvest",
        })
    }
}

#[cfg(test)]
mod tests {
    use super::DetachedEditorRuntimeGateway;
    use crate::core::gateway::{EditorRuntimeGateway, EditorRuntimeHighlightSet, GatewayError};
    use zircon_runtime_interface::ZrRuntimeViewportHandle;

    #[test]
    fn highlight_submission_reports_a_typed_missing_capability() {
        let error = DetachedEditorRuntimeGateway
            .submit_highlight_set(EditorRuntimeHighlightSet::new(
                ZrRuntimeViewportHandle::new(1),
                1,
                [4],
                true,
                [0.2, 0.6, 0.9, 1.0],
            ))
            .unwrap_err();

        assert_eq!(
            error,
            GatewayError::CapabilityMissing {
                capability: "runtime.editor_overlay.highlight_set",
            }
        );
    }
}
