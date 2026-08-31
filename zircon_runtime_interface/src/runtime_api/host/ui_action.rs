use std::fmt;

use serde::{Deserialize, Serialize};

use crate::handles::ZrRuntimeViewportHandle;
use crate::ui::component::UiSecureTextValueRef;
use crate::ui::dispatch::UiTemplateActionInvocation;
use crate::ui::event_ui::{UiNodeId, UiTreeId};

/// A secure-content-free Runtime UI action delivery for an application host.
///
/// Secure text is represented only by an opaque reference. Resolving that reference requires a
/// separate trusted-session contract; this request never carries the underlying text.
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct ZrRuntimeUiActionHostRequestV1 {
    pub target_viewport: ZrRuntimeViewportHandle,
    pub target_surface: u32,
    pub input_sequence: u64,
    pub action_index: u32,
    pub tree_id: UiTreeId,
    pub target: UiNodeId,
    pub invocation: UiTemplateActionInvocation,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub secure_value: Option<UiSecureTextValueRef>,
}

impl ZrRuntimeUiActionHostRequestV1 {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        target_viewport: ZrRuntimeViewportHandle,
        target_surface: u32,
        input_sequence: u64,
        action_index: u32,
        tree_id: UiTreeId,
        target: UiNodeId,
        invocation: UiTemplateActionInvocation,
        secure_value: Option<UiSecureTextValueRef>,
    ) -> Self {
        Self {
            target_viewport,
            target_surface,
            input_sequence,
            action_index,
            tree_id,
            target,
            invocation,
            secure_value,
        }
    }
}

impl fmt::Debug for ZrRuntimeUiActionHostRequestV1 {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("ZrRuntimeUiActionHostRequestV1")
            .field("target_viewport", &self.target_viewport)
            .field("target_surface", &self.target_surface)
            .field("input_sequence", &self.input_sequence)
            .field("action_index", &self.action_index)
            .field("tree_id", &self.tree_id)
            .field("target", &self.target)
            .field("action_target", &self.invocation.target_id())
            .field("secure_value", &self.secure_value.is_some())
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use super::*;
    use crate::ui::component::UiValue;

    #[test]
    fn secure_ui_action_delivery_serializes_identity_without_plaintext() {
        let tree_id = UiTreeId::new("secure.auth");
        let reference = UiSecureTextValueRef::issue(tree_id.clone(), UiNodeId::new(7), "value");
        let request = ZrRuntimeUiActionHostRequestV1::new(
            ZrRuntimeViewportHandle::new(3),
            5,
            11,
            0,
            tree_id,
            UiNodeId::new(7),
            UiTemplateActionInvocation::route(
                "woc.shell.auth.submit",
                BTreeMap::from([("credential".to_string(), UiValue::Null)]),
            ),
            Some(reference),
        );

        let encoded = serde_json::to_string(&request).expect("serialize secure UI action");
        assert!(encoded.contains("woc.shell.auth.submit"));
        assert!(!encoded.contains("correct horse battery staple"));

        let debug = format!("{request:?}");
        assert!(debug.contains("woc.shell.auth.submit"));
        assert!(!debug.contains("credential"));
    }
}
