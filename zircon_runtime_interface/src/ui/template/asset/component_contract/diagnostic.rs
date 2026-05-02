use serde::{Deserialize, Serialize};

use crate::ui::template::UiAssetError;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum UiComponentContractDiagnosticCode {
    InvalidPublicPart,
    PrivateSelector,
    ApiMismatch,
    ClosedRootClass,
    PrivateBindingTarget,
    PrivateFocusTarget,
}

impl UiComponentContractDiagnosticCode {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::InvalidPublicPart => "invalid_public_part",
            Self::PrivateSelector => "private_selector",
            Self::ApiMismatch => "api_mismatch",
            Self::ClosedRootClass => "closed_root_class",
            Self::PrivateBindingTarget => "private_binding_target",
            Self::PrivateFocusTarget => "private_focus_target",
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct UiComponentContractDiagnostic {
    pub code: UiComponentContractDiagnosticCode,
    pub message: String,
    pub path: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub target_node_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub target_control_id: Option<String>,
}

impl UiComponentContractDiagnostic {
    pub fn new(
        code: UiComponentContractDiagnosticCode,
        message: impl Into<String>,
        path: impl Into<String>,
    ) -> Self {
        Self {
            code,
            message: message.into(),
            path: path.into(),
            target_node_id: None,
            target_control_id: None,
        }
    }

    pub fn with_target_node_id(mut self, target_node_id: impl Into<String>) -> Self {
        self.target_node_id = Some(target_node_id.into());
        self
    }

    pub fn with_target_control_id(mut self, target_control_id: impl Into<String>) -> Self {
        self.target_control_id = Some(target_control_id.into());
        self
    }

    pub fn into_asset_error(self, asset_id: impl Into<String>) -> UiAssetError {
        UiAssetError::InvalidDocument {
            asset_id: asset_id.into(),
            detail: self.message,
        }
    }
}
