use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use crate::ui::accessibility::UiAccessibilityContract;
use crate::ui::binding::UiEventKind;
use crate::ui::component::UiComponentEventKind;
use crate::ui::focus::UiFocusContract;
use crate::ui::navigation::UiNavigationContract;
use crate::ui::picking::UiPickPolicy;
use crate::ui::widget::UiWidgetContract;

use super::{UiActionRef, UiBindingTargetAssignment};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
/// 写权限与触发时机分开建模，调用方据 mode 决定方向，而不会仅凭事件类型猜测双向写入。
pub struct UiBindingWritePermissions {
    pub writes_target: bool,
    pub writes_source: bool,
    pub publishes_command: bool,
}

impl UiBindingWritePermissions {
    pub const TARGET_ONLY: Self = Self {
        writes_target: true,
        writes_source: false,
        publishes_command: false,
    };
    pub const SOURCE_AND_TARGET: Self = Self {
        writes_target: true,
        writes_source: true,
        publishes_command: false,
    };
    pub const COMMAND_ONLY: Self = Self {
        writes_target: false,
        writes_source: false,
        publishes_command: true,
    };
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
/// 触发时机是绑定模式的稳定映射结果，覆盖实例化、源变化、事件和命令分发阶段。
pub enum UiBindingTriggerTiming {
    Instantiation,
    SourceChange,
    SourceOrTargetChange,
    EventDispatch,
    CommandDispatch,
}

/// 绑定模式定义触发时机和写入方向；默认事件模式兼容未声明模式的旧文档。
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum UiBindingMode {
    OneTime,
    OneWay,
    TwoWay,
    #[default]
    Event,
    Command,
}

impl UiBindingMode {
    pub const fn trigger_timing(self) -> UiBindingTriggerTiming {
        match self {
            Self::OneTime => UiBindingTriggerTiming::Instantiation,
            Self::OneWay => UiBindingTriggerTiming::SourceChange,
            Self::TwoWay => UiBindingTriggerTiming::SourceOrTargetChange,
            Self::Event => UiBindingTriggerTiming::EventDispatch,
            Self::Command => UiBindingTriggerTiming::CommandDispatch,
        }
    }

    pub const fn write_permissions(self) -> UiBindingWritePermissions {
        match self {
            Self::OneTime | Self::OneWay | Self::Event => UiBindingWritePermissions::TARGET_ONLY,
            Self::TwoWay => UiBindingWritePermissions::SOURCE_AND_TARGET,
            Self::Command => UiBindingWritePermissions::COMMAND_ONLY,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
/// 源绑定把事件、可选组件事件、动作路由和目标赋值组合为一项声明；编译器再生成运行时程序。
pub struct UiBindingRef {
    pub id: String,
    pub event: UiEventKind,
    #[serde(default)]
    pub mode: UiBindingMode,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub component_event: Option<UiComponentEventKind>,
    #[serde(default)]
    pub route: Option<String>,
    #[serde(default)]
    pub action: Option<UiActionRef>,
    #[serde(default)]
    pub targets: Vec<UiBindingTargetAssignment>,
}

/// 展开后的模板树节点；编译器据来源资产和逐绑定来源构建归属信息。
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct UiTemplateNode {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub source_asset_id: Option<String>,
    #[serde(default)]
    pub component: Option<String>,
    #[serde(default)]
    pub template: Option<String>,
    #[serde(default)]
    pub slot: Option<String>,
    #[serde(default)]
    pub control_id: Option<String>,
    #[serde(default)]
    pub classes: Vec<String>,
    #[serde(default)]
    pub bindings: Vec<UiBindingRef>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    /// 与 `bindings` 按索引对应；空数组时绑定沿用节点来源资产。
    pub binding_source_asset_ids: Vec<String>,
    #[serde(default)]
    pub children: Vec<UiTemplateNode>,
    #[serde(default)]
    pub slots: BTreeMap<String, Vec<UiTemplateNode>>,
    #[serde(default)]
    pub attributes: BTreeMap<String, toml::Value>,
    #[serde(default)]
    pub slot_attributes: BTreeMap<String, toml::Value>,
    #[serde(default)]
    pub style_overrides: BTreeMap<String, toml::Value>,
    #[serde(default)]
    pub style_tokens: BTreeMap<String, String>,
    #[serde(default)]
    pub focus: UiFocusContract,
    #[serde(default)]
    pub navigation: UiNavigationContract,
    #[serde(default)]
    pub picking: UiPickPolicy,
    #[serde(default)]
    pub a11y: UiAccessibilityContract,
    #[serde(default)]
    pub widget: UiWidgetContract,
}

#[cfg(test)]
#[path = "tests/document.rs"]
mod tests;
