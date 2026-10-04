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
    // EXEMPT(GEN-Q7): host action request construction mirrors the fixed ABI payload fields.
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
#[path = "tests/ui_action.rs"]
mod tests;
