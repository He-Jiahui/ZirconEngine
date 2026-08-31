use serde::{Deserialize, Serialize};

use crate::handles::ZrRuntimeViewportHandle;
use crate::ui::dispatch::{UiClipboardRequest, UiClipboardTransferId, UiClipboardTransferOutcome};
use crate::ui::event_ui::UiNodeId;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ZrRuntimeClipboardHostRequestV1 {
    pub target_viewport: ZrRuntimeViewportHandle,
    pub target_surface: u32,
    pub request: UiClipboardRequest,
}

impl ZrRuntimeClipboardHostRequestV1 {
    pub fn new(
        target_viewport: ZrRuntimeViewportHandle,
        target_surface: u32,
        request: UiClipboardRequest,
    ) -> Self {
        Self {
            target_viewport,
            target_surface,
            request,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ZrRuntimeClipboardResultV1 {
    pub target_surface: u32,
    pub transfer_id: UiClipboardTransferId,
    pub owner: UiNodeId,
    pub outcome: UiClipboardTransferOutcome,
}

impl ZrRuntimeClipboardResultV1 {
    pub fn new(
        target_surface: u32,
        transfer_id: UiClipboardTransferId,
        owner: UiNodeId,
        outcome: UiClipboardTransferOutcome,
    ) -> Self {
        Self {
            target_surface,
            transfer_id,
            owner,
            outcome,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ui::dispatch::{
        UiClipboardRequestKind, UiClipboardTransferIntent, UiClipboardTransferOutcome,
    };
    use crate::{
        ZrRuntimeHostRequestBatchV1, ZrRuntimeHostRequestV1, ZIRCON_RUNTIME_ABI_VERSION_V1,
        ZR_RUNTIME_CLIPBOARD_RESULT_REQUEST_LIMIT_V1,
        ZR_RUNTIME_CLIPBOARD_TEXT_MAX_ENCODED_BYTES_V1, ZR_RUNTIME_HOST_REQUEST_OUTPUT_LIMIT_V1,
    };

    #[test]
    fn clipboard_host_contract_roundtrips_surface_identity_and_typed_outcome() {
        let request = UiClipboardRequest {
            transfer_id: UiClipboardTransferId::issue(),
            intent: UiClipboardTransferIntent::Paste,
            expected_edit_revision: 7,
            kind: UiClipboardRequestKind::ReadText,
            owner: UiNodeId::new(41),
            text: None,
        };
        let host_request = ZrRuntimeClipboardHostRequestV1::new(
            ZrRuntimeViewportHandle::new(3),
            5,
            request.clone(),
        );
        let encoded = serde_json::to_vec(&host_request).expect("encode clipboard host request");
        assert_eq!(
            serde_json::from_slice::<ZrRuntimeClipboardHostRequestV1>(&encoded)
                .expect("decode clipboard host request"),
            host_request
        );

        let result = ZrRuntimeClipboardResultV1::new(
            host_request.target_surface,
            request.transfer_id,
            request.owner,
            UiClipboardTransferOutcome::ReadText {
                text: "paste".to_string(),
            },
        );
        let encoded = serde_json::to_vec(&result).expect("encode clipboard result");
        assert_eq!(
            serde_json::from_slice::<ZrRuntimeClipboardResultV1>(&encoded)
                .expect("decode clipboard result"),
            result
        );
    }

    #[test]
    fn clipboard_body_budget_fits_worst_case_json_control_character_expansion() {
        let body = "\0".repeat(ZR_RUNTIME_CLIPBOARD_TEXT_MAX_ENCODED_BYTES_V1);
        let request = UiClipboardRequest {
            transfer_id: UiClipboardTransferId::issue(),
            intent: UiClipboardTransferIntent::Copy,
            expected_edit_revision: 11,
            kind: UiClipboardRequestKind::WriteText,
            owner: UiNodeId::new(41),
            text: Some(body.clone()),
        };
        let host_batch = ZrRuntimeHostRequestBatchV1::new(
            ZIRCON_RUNTIME_ABI_VERSION_V1,
            vec![ZrRuntimeHostRequestV1::clipboard(
                ZrRuntimeClipboardHostRequestV1::new(
                    ZrRuntimeViewportHandle::new(3),
                    5,
                    request.clone(),
                ),
            )],
        );
        let encoded_request = serde_json::to_vec(&host_batch).expect("encode clipboard host batch");
        assert!(encoded_request.len() <= ZR_RUNTIME_HOST_REQUEST_OUTPUT_LIMIT_V1.max_encoded_bytes);

        let result = ZrRuntimeClipboardResultV1::new(
            5,
            request.transfer_id,
            request.owner,
            UiClipboardTransferOutcome::ReadText { text: body },
        );
        let encoded_result = serde_json::to_vec(&result).expect("encode clipboard result");
        assert!(
            encoded_result.len() <= ZR_RUNTIME_CLIPBOARD_RESULT_REQUEST_LIMIT_V1.max_encoded_bytes
        );
    }
}
