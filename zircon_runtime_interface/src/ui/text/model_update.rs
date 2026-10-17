use core::fmt;

use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::ui::event_ui::{UiNodeId, UiTreeId};

use super::{UiTextDocumentId, UiTextDocumentRevision, UiTextEditReceipt};

pub const UI_TEXT_MODEL_UPDATE_SCHEMA_VERSION: u32 = 1;

/// Opaque identity used to correlate an update with its immediate or deferred receipt.
#[derive(
    Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize,
)]
#[serde(transparent)]
pub struct UiTextModelUpdateId(Uuid);

impl UiTextModelUpdateId {
    pub fn issue() -> Self {
        Self(Uuid::new_v4())
    }

    pub fn is_valid(self) -> bool {
        !self.0.is_nil()
    }
}

/// Compare-and-swap key for one retained editable-text document revision.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct UiTextDocumentKey {
    pub document_id: UiTextDocumentId,
    pub revision: UiTextDocumentRevision,
}

/// Declares whether an update may defer while the owner has editing focus.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum UiTextModelUpdateOrigin {
    /// A binding/model refresh that must preserve a focused edit buffer.
    #[default]
    BoundRefresh,
    /// An explicit API replacement that applies even while focused.
    ExplicitSetText,
    /// An explicit load/reload replacement that applies even while focused.
    ExplicitLoadText,
}

/// Versioned model-to-edit-session update qualified by the expected document revision.
///
/// The value may contain secure text. Consumers must not log the request with an alternate
/// formatter or retain the value after the request reaches a terminal state.
///
/// # 约束与生命周期
/// 请求持有 `value` 字符串；构造与派生反序列化均不自动调用本类型的 `validate`。
/// 自定义 Debug 只输出内容字节数，Serde 仍编码完整内容；调试隐藏不代表清零或保密。
/// 文档键为值快照，实际树归属、修订匹配及内容准入由 Runtime 判断。
#[derive(Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct UiTextModelUpdateRequest {
    pub schema_version: u32,
    pub request_id: UiTextModelUpdateId,
    pub tree_id: UiTreeId,
    pub node_id: UiNodeId,
    pub expected_document: UiTextDocumentKey,
    pub origin: UiTextModelUpdateOrigin,
    pub value: String,
}

impl UiTextModelUpdateRequest {
    /// 组装更新请求并生成关联标识，交给文档所有者执行后续准入。
    ///
    /// # 参数
    /// - `tree_id`、`node_id`：预期接收更新的树与节点。
    /// - `expected_document`：调用者观察到的文档身份与修订快照。
    /// - `origin`：更新意图，由 Runtime 据此选择聚焦时的处理方式。
    /// - `value`：转为请求持有的字符串，未在这里检查内容策略或大小。
    ///
    /// # 返回
    /// 尚未执行 [`Self::validate`] 或应用到文档的请求。
    pub fn new(
        tree_id: UiTreeId,
        node_id: UiNodeId,
        expected_document: UiTextDocumentKey,
        origin: UiTextModelUpdateOrigin,
        value: impl Into<String>,
    ) -> Self {
        Self {
            schema_version: UI_TEXT_MODEL_UPDATE_SCHEMA_VERSION,
            request_id: UiTextModelUpdateId::issue(),
            tree_id,
            node_id,
            expected_document,
            origin,
            value: value.into(),
        }
    }

    /// 只验证协议版本和请求标识；树归属、当前修订及内容策略由拥有文档的运行时判断。
    /// 另要求预期文档标识非 nil；这里只核对身份格式，不查询树或文档是否存在。
    pub fn validate(&self) -> Result<(), UiTextModelUpdateFailure> {
        if self.schema_version != UI_TEXT_MODEL_UPDATE_SCHEMA_VERSION {
            return Err(UiTextModelUpdateFailure::UnsupportedSchemaVersion);
        }
        if !self.request_id.is_valid() {
            return Err(UiTextModelUpdateFailure::InvalidRequestId);
        }
        if !self.expected_document.document_id.is_valid() {
            return Err(UiTextModelUpdateFailure::InvalidDocumentId);
        }
        Ok(())
    }
}

impl fmt::Debug for UiTextModelUpdateRequest {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("UiTextModelUpdateRequest")
            .field("schema_version", &self.schema_version)
            .field("request_id", &self.request_id)
            .field("tree_id", &self.tree_id)
            .field("node_id", &self.node_id)
            .field("expected_document", &self.expected_document)
            .field("origin", &self.origin)
            .field("value_byte_len", &self.value.len())
            .finish()
    }
}

/// Outcome of an immediate request or a deferred focus-loss resolution.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum UiTextModelUpdateStatus {
    Applied,
    Unchanged,
    Deferred,
    Conflict,
    #[default]
    Rejected,
}

/// Stable fail-closed reason carried by a content-free model update receipt.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum UiTextModelUpdateFailure {
    UnsupportedSchemaVersion,
    InvalidRequestId,
    InvalidDocumentId,
    WrongTree,
    InvalidTarget,
    DocumentUnavailable,
    StaleDocument,
    PendingQueueFull,
    ValueTooLarge,
    PendingBytesExceeded,
    Superseded,
    SecurityPolicyChanged,
    SecureValueUnavailable,
    PropertyRejected,
    DocumentRejected,
    OwnerDetached,
}

/// Content-free acknowledgement for a model update request.
///
/// `Deferred` is returned directly by the update call. Its eventual terminal receipt is drained
/// from the manager after focus loss, supersession, owner teardown, or policy change.
///
/// # 约束与生命周期
/// 延后状态表示尚未应用。Runtime 的 drain 只移出已产生的终态回执，不处理待办或触发失焦；
/// 管理器销毁后也不保证仍可取得终态。本值不保有文档，实际结果以生产者提交为准。
/// 外层派生反序列化不调用本类型的 `validate`，但嵌套编辑回执仍执行自身的读取校验。
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct UiTextModelUpdateReceipt {
    pub schema_version: u32,
    pub request_id: UiTextModelUpdateId,
    pub tree_id: UiTreeId,
    pub node_id: UiNodeId,
    pub origin: UiTextModelUpdateOrigin,
    pub status: UiTextModelUpdateStatus,
    pub expected_document: UiTextDocumentKey,
    pub current_document: Option<UiTextDocumentKey>,
    pub document_edit: Option<UiTextEditReceipt>,
    pub failure: Option<UiTextModelUpdateFailure>,
}

impl UiTextModelUpdateReceipt {
    /// 校验回执字段之间的一致性；实际应用结果仍以运行时返回的状态和修订为准。
    pub fn validate(&self) -> Result<(), UiTextModelUpdateFailure> {
        if self.schema_version != UI_TEXT_MODEL_UPDATE_SCHEMA_VERSION {
            return Err(UiTextModelUpdateFailure::UnsupportedSchemaVersion);
        }
        // 拒绝回执需要报告坏输入；较早准入失败可保留原始坏标识，合法标识则不能伪造同类失败。
        let rejects_unsupported_schema = self.status == UiTextModelUpdateStatus::Rejected
            && self.failure == Some(UiTextModelUpdateFailure::UnsupportedSchemaVersion);
        let rejects_invalid_request_id = self.status == UiTextModelUpdateStatus::Rejected
            && self.failure == Some(UiTextModelUpdateFailure::InvalidRequestId);
        let rejects_invalid_document_id = self.status == UiTextModelUpdateStatus::Rejected
            && self.failure == Some(UiTextModelUpdateFailure::InvalidDocumentId);
        if (!self.request_id.is_valid()
            && !(rejects_unsupported_schema || rejects_invalid_request_id))
            || (self.request_id.is_valid() && rejects_invalid_request_id)
        {
            return Err(UiTextModelUpdateFailure::InvalidRequestId);
        }
        if (!self.expected_document.document_id.is_valid()
            && !(rejects_unsupported_schema
                || rejects_invalid_request_id
                || rejects_invalid_document_id))
            || (self.expected_document.document_id.is_valid() && rejects_invalid_document_id)
        {
            return Err(UiTextModelUpdateFailure::InvalidDocumentId);
        }
        if self
            .current_document
            .is_some_and(|key| !key.document_id.is_valid())
        {
            return Err(UiTextModelUpdateFailure::InvalidDocumentId);
        }
        if !matches!(self.status, UiTextModelUpdateStatus::Rejected)
            && self.current_document.is_none()
        {
            return Err(UiTextModelUpdateFailure::DocumentRejected);
        }
        let valid_status = match self.status {
            UiTextModelUpdateStatus::Applied
            | UiTextModelUpdateStatus::Unchanged
            | UiTextModelUpdateStatus::Deferred => self.failure.is_none(),
            UiTextModelUpdateStatus::Conflict => {
                self.failure == Some(UiTextModelUpdateFailure::StaleDocument)
            }
            UiTextModelUpdateStatus::Rejected => self.failure.is_some(),
        };
        if !valid_status {
            return Err(UiTextModelUpdateFailure::DocumentRejected);
        }
        if let Some(document_edit) = &self.document_edit {
            document_edit
                .validate()
                .map_err(|_| UiTextModelUpdateFailure::DocumentRejected)?;
            if self.status != UiTextModelUpdateStatus::Applied
                || document_edit.node_id != self.node_id
                || document_edit.document_id != self.expected_document.document_id
                || document_edit.previous_revision != self.expected_document.revision
                || self.current_document
                    != Some(UiTextDocumentKey {
                        document_id: document_edit.document_id,
                        revision: document_edit.revision,
                    })
            {
                return Err(UiTextModelUpdateFailure::DocumentRejected);
            }
        }
        Ok(())
    }
}

#[cfg(test)]
#[path = "tests/model_update.rs"]
mod tests;
