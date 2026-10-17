//! 事件信封把控件身份、目标和类型化事件送往宿主适配器，并校验反序列化后的事件种类。

use serde::{Deserialize, Serialize};

use super::UiComponentBindingTarget;
use crate::ui::component::{UiComponentEvent, UiComponentEventKind, UiDragSourceMetadata};

/// Wraps a typed component event with enough target metadata for host adapters.
///
/// # 使用与生命周期
/// 事件生产者把目标和类型化载荷交给信封，再由宿主解析目标及应用事件；所有字段均由信封拥有。
/// [`Self::new`] 从载荷计算种类，反序列化的显式转换会拒绝种类与载荷不一致的线格式。
/// 字段公开，手工修改后仍由调用方维护这一关系；文档、控件、主体存在性与域权限由宿主校验。
/// 此值不借用保留树，也不延长目标对象的生命周期。
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(try_from = "UiComponentEventEnvelopeWire")]
pub struct UiComponentEventEnvelope {
    pub document_id: String,
    pub control_id: String,
    pub component_id: Option<String>,
    pub target: UiComponentBindingTarget,
    pub event_kind: UiComponentEventKind,
    pub event: UiComponentEvent,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub source: Option<UiDragSourceMetadata>,
}

/// 反序列化的临时自有载体；先完整接收字段，再经 [`UiComponentEventEnvelope`] 的转换检查种类关系。
#[derive(Deserialize)]
struct UiComponentEventEnvelopeWire {
    document_id: String,
    control_id: String,
    component_id: Option<String>,
    target: UiComponentBindingTarget,
    event_kind: UiComponentEventKind,
    event: UiComponentEvent,
    source: Option<UiDragSourceMetadata>,
}

impl TryFrom<UiComponentEventEnvelopeWire> for UiComponentEventEnvelope {
    type Error = String;

    /// 消费临时载体；种类分歧时返回诊断字符串，否则把全部字段移入信封。
    /// 此处只增加种类关系检查，不查询文档目标、执行事件或产生宿主写入。
    fn try_from(value: UiComponentEventEnvelopeWire) -> Result<Self, Self::Error> {
        // 线格式同时携带声明种类和类型化载荷，必须防止两者分叉。
        let actual_event_kind = value.event.kind();
        if value.event_kind != actual_event_kind {
            return Err(format!(
                "component event kind {:?} does not match typed event {:?}",
                value.event_kind, actual_event_kind
            ));
        }

        Ok(Self {
            document_id: value.document_id,
            control_id: value.control_id,
            component_id: value.component_id,
            target: value.target,
            event_kind: value.event_kind,
            event: value.event,
            source: value.source,
        })
    }
}

impl UiComponentEventEnvelope {
    /// 接管目标和事件，创建种类由载荷推导的信封。
    ///
    /// # 参数与返回
    /// `document_id`、`control_id` 转换为自有身份文字，`target` 与 `event` 按值移入。
    /// 返回组件标识和拖拽来源均为空的值；构造时不查找文档、控件或宿主域。
    /// 可用 [`Self::with_component_id`] 和 [`Self::with_source`] 补充来源信息。
    pub fn new(
        document_id: impl Into<String>,
        control_id: impl Into<String>,
        target: UiComponentBindingTarget,
        event: UiComponentEvent,
    ) -> Self {
        let event_kind = event.kind();
        Self {
            document_id: document_id.into(),
            control_id: control_id.into(),
            component_id: None,
            target,
            event_kind,
            event,
            source: None,
        }
    }

    /// 接管组件标识并替换旧值，返回信封；不查组件目录或改变载荷种类。
    pub fn with_component_id(mut self, component_id: impl Into<String>) -> Self {
        self.component_id = Some(component_id.into());
        self
    }

    /// 接管拖拽来源快照并替换旧值，返回信封；不借用或延长来源对象生命周期。
    pub fn with_source(mut self, source: UiDragSourceMetadata) -> Self {
        self.source = Some(source);
        self
    }
}
