//! UI 文本编辑、模型更新与富链接在边界上传递的中立值契约。
//!
//! Runtime 保有实际文档及编辑状态；这里的标识、修订和选择只是快照，不延长文档生命。
//! 接口校验与拥有文档的 Runtime 准入分工见各构造器和校验方法。

use core::fmt;

use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::ui::event_ui::UiNodeId;
use crate::ui::surface::{UiTextByteRange, UiTextCaretAffinity};

#[path = "text/model_update.rs"]
mod model_update;
#[path = "text/number_model_update.rs"]
mod number_model_update;
#[path = "text/rich_link_target.rs"]
mod rich_link_target;

pub use model_update::{
    UiTextDocumentKey, UiTextModelUpdateFailure, UiTextModelUpdateId, UiTextModelUpdateOrigin,
    UiTextModelUpdateReceipt, UiTextModelUpdateRequest, UiTextModelUpdateStatus,
    UI_TEXT_MODEL_UPDATE_SCHEMA_VERSION,
};
pub use number_model_update::{
    UiNumberModelId, UiNumberModelKey, UiNumberModelRevision, UiNumberModelUpdateFailure,
    UiNumberModelUpdateId, UiNumberModelUpdateOrigin, UiNumberModelUpdateReceipt,
    UiNumberModelUpdateRequest, UiNumberModelUpdateStatus, UI_NUMBER_MODEL_UPDATE_SCHEMA_VERSION,
};
pub use rich_link_target::{UiRichLinkTarget, UiRichLinkTargetError};

pub const UI_TEXT_EDIT_RECEIPT_SCHEMA_VERSION: u32 = 1;

/// 文本生产者使用的不透明文档标识；默认值为 nil，`issue` 只生成标识而不创建文档。
/// `is_valid` 仅检查非 nil，不查询文档存在性，也不保有 Runtime 文档。
#[derive(
    Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize,
)]
#[serde(transparent)]
pub struct UiTextDocumentId(Uuid);

impl UiTextDocumentId {
    pub fn issue() -> Self {
        Self(Uuid::new_v4())
    }

    pub fn is_valid(self) -> bool {
        !self.0.is_nil()
    }
}

/// 文档修订的值快照；`new` 仅包装计数，不确认它是否为实际文档的当前修订。
/// [`Self::checked_next`] 在计数溢出时返回 `None`，由生产者决定如何拒绝后续编辑。
#[derive(
    Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize,
)]
#[serde(transparent)]
pub struct UiTextDocumentRevision(u64);

impl UiTextDocumentRevision {
    pub const fn new(value: u64) -> Self {
        Self(value)
    }

    pub const fn get(self) -> u64 {
        self.0
    }

    pub const fn checked_next(self) -> Option<Self> {
        match self.0.checked_add(1) {
            Some(value) => Some(Self(value)),
            None => None,
        }
    }
}

/// 生产者报告的编辑来源标签；该值本身不验证事件来源或执行输入操作。
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum UiTextEditSource {
    #[default]
    Keyboard,
    Pointer,
    Ime,
    Clipboard,
    Accessibility,
    Programmatic,
}

/// 文本编辑的分类标签；实际编辑、撤销与重做事务由 Runtime 完成。
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum UiTextEditKind {
    Insert,
    Delete,
    Replace,
    CompositionCommit,
    Undo,
    Redo,
}

/// 一次编辑在旧文档与新文档中的字节区间快照；两侧长度可以不同。
/// 区间是否落在对应文档内，须由掌握文档内容的生产者另行核验。
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct UiTextChangedRanges {
    pub old: UiTextByteRange,
    pub new: UiTextByteRange,
}

/// 编辑后的锚点、焦点与焦点亲和性快照，位置按文档 UTF-8 字节计。
/// 构造折叠选择只令两个端点相同，不检查文档长度或字符边界。
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct UiTextByteSelection {
    pub anchor_byte: u32,
    pub focus_byte: u32,
    pub focus_affinity: UiTextCaretAffinity,
}

impl UiTextByteSelection {
    pub const fn collapsed(offset: u32) -> Self {
        Self::collapsed_with_affinity(offset, UiTextCaretAffinity::Downstream)
    }

    /// 按给定亲和性构造折叠选择，供生产者填入编辑后的选择位置。
    ///
    /// # 参数与返回
    /// `offset` 是未经文档校验的字节位置；返回值的锚点与焦点均为该位置。
    /// `focus_affinity` 原样保存，位置有效性由掌握文档的调用者保证。
    pub const fn collapsed_with_affinity(offset: u32, focus_affinity: UiTextCaretAffinity) -> Self {
        Self {
            anchor_byte: offset,
            focus_byte: offset,
            focus_affinity,
        }
    }
}

/// 报告一次文本编辑的版本化值快照，不持有实际文档或证明编辑已提交。
///
/// Serde 读取经私有 wire 转换和 [`Self::validate`]；直接组装或序列化本值不会自动校验。
/// [`Self::new`] 从前一修订建立相邻后继，具体检查边界见 [`Self::validate`]。
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(try_from = "UiTextEditReceiptWire")]
pub struct UiTextEditReceipt {
    pub schema_version: u32,
    pub node_id: UiNodeId,
    pub document_id: UiTextDocumentId,
    pub previous_revision: UiTextDocumentRevision,
    pub revision: UiTextDocumentRevision,
    pub source: UiTextEditSource,
    pub kind: UiTextEditKind,
    pub changed: UiTextChangedRanges,
    pub selection: UiTextByteSelection,
}

#[derive(Deserialize)]
struct UiTextEditReceiptWire {
    schema_version: u32,
    node_id: UiNodeId,
    document_id: UiTextDocumentId,
    previous_revision: UiTextDocumentRevision,
    revision: UiTextDocumentRevision,
    source: UiTextEditSource,
    kind: UiTextEditKind,
    changed: UiTextChangedRanges,
    selection: UiTextByteSelection,
}

impl TryFrom<UiTextEditReceiptWire> for UiTextEditReceipt {
    type Error = UiTextEditReceiptError;

    fn try_from(wire: UiTextEditReceiptWire) -> Result<Self, Self::Error> {
        let receipt = Self {
            schema_version: wire.schema_version,
            node_id: wire.node_id,
            document_id: wire.document_id,
            previous_revision: wire.previous_revision,
            revision: wire.revision,
            source: wire.source,
            kind: wire.kind,
            changed: wire.changed,
            selection: wire.selection,
        };
        receipt.validate()?;
        Ok(receipt)
    }
}

impl UiTextEditReceipt {
    /// 从前一修订构造相邻后继回执，并执行接口字段校验。
    ///
    /// # 参数
    /// - `node_id`、`document_id`：生产者报告的节点与文档身份，不触发存储查询。
    /// - `previous_revision`：编辑前修订；返回修订取其加一。
    /// - `source`、`kind`：编辑来源与分类标签。
    /// - `changed`、`selection`：旧/新字节区间和编辑后的选择快照。
    ///
    /// # 返回
    /// 成功返回带当前协议版本的回执；修订溢出或 [`Self::validate`] 失败时返回错误。
    // EXEMPT(GEN-Q7): text edit receipt construction preserves the fixed UI/runtime ABI field set.
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        node_id: UiNodeId,
        document_id: UiTextDocumentId,
        previous_revision: UiTextDocumentRevision,
        source: UiTextEditSource,
        kind: UiTextEditKind,
        changed: UiTextChangedRanges,
        selection: UiTextByteSelection,
    ) -> Result<Self, UiTextEditReceiptError> {
        let revision = previous_revision
            .checked_next()
            .ok_or(UiTextEditReceiptError::RevisionExhausted)?;
        let receipt = Self {
            schema_version: UI_TEXT_EDIT_RECEIPT_SCHEMA_VERSION,
            node_id,
            document_id,
            previous_revision,
            revision,
            source,
            kind,
            changed,
            selection,
        };
        receipt.validate()?;
        Ok(receipt)
    }

    /// 核对接口字段之间的一致性，供构造器、反序列化或直接组装后的调用者使用。
    ///
    /// # 返回与约束
    /// `Ok(())` 仅表示版本受支持、文档标识非 nil、修订相邻且旧/新区间各自有序。
    /// 本方法不读取文档，不检查文档长度、选择端点或 UTF-8 字符边界，也不确认提交结果。
    pub fn validate(&self) -> Result<(), UiTextEditReceiptError> {
        if self.schema_version != UI_TEXT_EDIT_RECEIPT_SCHEMA_VERSION {
            return Err(UiTextEditReceiptError::UnsupportedSchemaVersion);
        }
        if !self.document_id.is_valid() {
            return Err(UiTextEditReceiptError::InvalidDocumentId);
        }
        let Some(expected_revision) = self.previous_revision.checked_next() else {
            return Err(UiTextEditReceiptError::RevisionExhausted);
        };
        if expected_revision != self.revision {
            return Err(UiTextEditReceiptError::NonConsecutiveRevision);
        }
        if self.changed.old.start_byte > self.changed.old.end_byte {
            return Err(UiTextEditReceiptError::InvalidOldRange);
        }
        if self.changed.new.start_byte > self.changed.new.end_byte {
            return Err(UiTextEditReceiptError::InvalidNewRange);
        }
        Ok(())
    }
}

/// 编辑回执构造或显式校验的失败分类；Serde 读取会将这些错误转成解码错误。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum UiTextEditReceiptError {
    UnsupportedSchemaVersion,
    InvalidDocumentId,
    RevisionExhausted,
    NonConsecutiveRevision,
    InvalidOldRange,
    InvalidNewRange,
}

impl fmt::Display for UiTextEditReceiptError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::UnsupportedSchemaVersion => "unsupported text edit receipt schema version",
            Self::InvalidDocumentId => "text edit receipt has an invalid document id",
            Self::RevisionExhausted => "text edit receipt revision is exhausted",
            Self::NonConsecutiveRevision => "text edit receipt revision is not consecutive",
            Self::InvalidOldRange => "text edit receipt old range is out of order",
            Self::InvalidNewRange => "text edit receipt new range is out of order",
        })
    }
}

impl std::error::Error for UiTextEditReceiptError {}

/// 可序列化的光标显示偏好；读取缺失字段时采用 [`Default`]。
/// 本载体保留宽度、颜色文本和闪烁周期，不在接口层解析颜色或校验数值可用性。
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct UiTextCursorStyle {
    pub width: f32,
    pub color: Option<String>,
    pub blink_period_millis: Option<u64>,
    pub visible: bool,
}

impl Default for UiTextCursorStyle {
    fn default() -> Self {
        Self {
            width: 1.0,
            color: None,
            blink_period_millis: None,
            visible: true,
        }
    }
}
