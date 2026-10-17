use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use crate::ui::template::UiCompiledBindingHandle;
use crate::ui::{
    binding::UiEventKind,
    component::{
        UiComponentBindingTarget, UiComponentEvent, UiComponentEventEnvelope, UiDragMetrics,
        UiValue,
    },
    event_ui::{UiNodeId, UiTreeId},
};

/// 模板绑定产出的动作身份：声明的 action 与操作 route 分别编码，Editor 回调按种类分发到命令或操作。
///
/// # 约束与所有权
/// 记录拥有目标标识和参数；构造器不查找目标、不验证标识或执行回调。
/// 消费方仍须结合 `is_action` 与 `target_id` 选择命令或操作路由，并判断参数是否适配。
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct UiTemplateActionInvocation {
    target: UiTemplateActionTarget,
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub payload: BTreeMap<String, UiValue>,
}

impl UiTemplateActionInvocation {
    /// Constructs a parameterized local or operation route.
    ///
    /// # 参数
    /// `route` 转为拥有的标识；`payload` 按具名参数移入记录，不验证目标存在或参数是否适配。
    ///
    /// # 返回值
    /// 返回 Route 类目标的载体；空参数表在序列化时省略，缺少整个参数表字段时读取为空表。
    ///
    /// # 使用时机与参考
    /// 供组件绑定投影在决定路由及参数后使用；目标解析和执行由消费者完成。
    /// 无参数命令使用 [`Self::action`]。
    pub fn route(route: impl Into<String>, payload: BTreeMap<String, UiValue>) -> Self {
        Self {
            target: UiTemplateActionTarget::route(route),
            payload,
        }
    }

    /// 构造 Action 类目标，供消费方选择命令分支。
    ///
    /// # 参数与返回值
    /// `action` 转为拥有的标识；返回时参数表为空，构造时不查找或执行该命令。
    ///
    /// # 约束与参考
    /// Editor 的命令消费路径要求参数为空；公开参数表仍可被调用者修改，构造器不保持该约束。
    /// 携带具名参数的路由使用 [`Self::route`]。
    pub fn action(action: impl Into<String>) -> Self {
        Self {
            target: UiTemplateActionTarget::action(action),
            payload: BTreeMap::new(),
        }
    }

    pub fn target_id(&self) -> &str {
        &self.target.id
    }

    pub fn is_action(&self) -> bool {
        self.target.kind == UiTemplateActionTargetKind::Action
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
struct UiTemplateActionTarget {
    kind: UiTemplateActionTargetKind,
    id: String,
}

impl UiTemplateActionTarget {
    pub fn action(id: impl Into<String>) -> Self {
        Self {
            kind: UiTemplateActionTargetKind::Action,
            id: id.into(),
        }
    }

    pub fn route(id: impl Into<String>) -> Self {
        Self {
            kind: UiTemplateActionTargetKind::Route,
            id: id.into(),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
enum UiTemplateActionTargetKind {
    Action,
    Route,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum UiPointerComponentEventReason {
    #[default]
    DirectBinding,
    DefaultClick,
    DefaultDoubleClick,
    DefaultClickRejected,
    HoverEnter,
    HoverLeave,
    PressBegin,
    PressEnd,
    FocusGained,
    FocusLost,
    ScrollFallback,
}

/// 指针产生的组件事件及其绑定来源，供 UiSurface 和宿主层继续传递。
///
/// # 约束与生命周期
/// 本记录拥有封套和绑定标识，不借用创建时的 tree 或字符串；构造它不执行绑定或模板动作。
/// 节点、绑定和可选编译句柄是否仍有效，由使用它们的消费者判断。
/// `compiled_binding`、`drag`、`template_action` 缺失时恢复为 `None`，为 `None` 时不输出对应键。
///
/// # 参考
/// [`UiComponentEventEnvelope`]、[`UiTemplateActionInvocation`]。
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct UiPointerComponentEvent {
    pub node_id: UiNodeId,
    pub binding_id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub compiled_binding: Option<UiCompiledBindingHandle>,
    pub event_kind: UiEventKind,
    pub reason: UiPointerComponentEventReason,
    pub envelope: UiComponentEventEnvelope,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub drag: Option<UiDragMetrics>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub template_action: Option<UiTemplateActionInvocation>,
}

impl UiPointerComponentEvent {
    /// 组装基础事件；拖拽量、编译绑定与模板动作由调用者按来源补充。
    ///
    /// # 参数与返回值
    /// `tree_id` 的字符串被克隆到封套；`control_id` 同时用于控件身份和初始 showcase 目标。
    /// `node_id` 按值保留为发起节点；本构造器不检查它是否属于指定树。
    /// `binding_id` 标识触发绑定，`event_kind`、`event` 与 `reason` 分别保留分类、载荷及产生原因。
    /// 返回值尚未附加拖拽量、编译绑定或模板动作。
    ///
    /// # 参考
    /// [`Self::with_drag_metrics`]、[`Self::with_compiled_binding`]、[`Self::with_template_action`]。
    pub fn new(
        tree_id: &UiTreeId,
        node_id: UiNodeId,
        control_id: impl Into<String>,
        binding_id: impl Into<String>,
        event_kind: UiEventKind,
        event: UiComponentEvent,
        reason: UiPointerComponentEventReason,
    ) -> Self {
        let control_id = control_id.into();
        Self {
            node_id,
            binding_id: binding_id.into(),
            compiled_binding: None,
            event_kind,
            reason,
            envelope: UiComponentEventEnvelope::new(
                tree_id.0.clone(),
                control_id.as_str(),
                UiComponentBindingTarget::showcase(control_id.as_str()),
                event,
            ),
            drag: None,
            template_action: None,
        }
    }

    pub fn with_drag_metrics(mut self, drag: UiDragMetrics) -> Self {
        self.drag = Some(drag);
        self
    }

    pub fn with_compiled_binding(mut self, handle: UiCompiledBindingHandle) -> Self {
        self.compiled_binding = Some(handle);
        self
    }

    pub fn with_template_action(mut self, template_action: UiTemplateActionInvocation) -> Self {
        self.template_action = Some(template_action);
        self
    }
}

#[cfg(test)]
#[path = "tests/component_event.rs"]
mod tests;
