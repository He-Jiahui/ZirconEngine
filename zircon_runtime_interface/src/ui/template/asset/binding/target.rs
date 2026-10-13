//! 绑定结果的目标端点和表达式无值时的策略。
//! Runtime 编译阶段验证目标和值种类，surface dispatcher 在写入属性或发送动作时应用该策略。
use serde::{Deserialize, Serialize};

use crate::ui::component::{UiValue, UiValueKind};

/// 目标值缺失时的处理策略；Default 和 Fallback 在此都提供替代值。
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(tag = "policy", rename_all = "snake_case")]
pub enum UiBindingMissingValuePolicy {
    #[default]
    /// 没有表达式值时将该绑定报告为必需值缺失。
    Required,
    /// 没有表达式值时省略此目标更新。
    Optional,
    /// 没有表达式值时使用配置值；本解析层与 Fallback 采用相同替代行为。
    Default {
        value: UiValue,
    },
    /// 没有表达式值时使用配置值；策略名称在序列化协议中保留。
    Fallback {
        value: UiValue,
    },
    /// 没有表达式值时产生显式错误结果。
    Error,
}

#[derive(Clone, Debug, PartialEq)]
/// `resolve` 对调用端给出的缺值决策；不包含属性写入或动作派发副作用。
pub enum UiBindingMissingValueResolution {
    /// 表达式值或策略替代值，交给下游目标类型校验。
    Value(UiValue),
    /// 可选目标不产生更新。
    Omitted,
    /// 必需目标没有可用值。
    RequiredMissing,
    /// `Error` 策略要求调用端走失败路径。
    ExplicitError,
}

impl UiBindingMissingValuePolicy {
    /// 已求得的值总是优先；策略只在表达式没有返回值时生效。
    ///
    /// # 参数
    /// `value` 是表达式求值结果，None 才触发策略。
    ///
    /// # 返回值
    /// 返回值、忽略、必需缺失或显式错误之一；调用端负责写入/诊断。
    pub fn resolve(&self, value: Option<UiValue>) -> UiBindingMissingValueResolution {
        if let Some(value) = value {
            return UiBindingMissingValueResolution::Value(value);
        }
        match self {
            Self::Required => UiBindingMissingValueResolution::RequiredMissing,
            Self::Optional => UiBindingMissingValueResolution::Omitted,
            Self::Default { value } | Self::Fallback { value } => {
                UiBindingMissingValueResolution::Value(value.clone())
            }
            Self::Error => UiBindingMissingValueResolution::ExplicitError,
        }
    }

    /// 借用策略中配置的替代值；结果生命周期不超过该策略。
    pub fn substitute(&self) -> Option<&UiValue> {
        match self {
            Self::Default { value } | Self::Fallback { value } => Some(value),
            Self::Required | Self::Optional | Self::Error => None,
        }
    }

    /// 检查策略携带的替代值是否满足 `UiValue::is_finite`。
    pub fn is_well_formed(&self) -> bool {
        self.substitute().is_none_or(UiValue::is_finite)
    }

    /// 供 serde 判断默认 Required 是否可从源文档省略。
    pub(crate) fn is_required(&self) -> bool {
        matches!(self, Self::Required)
    }
}

/// 绑定赋值把源表达式与运行时拥有的目标表面配对；校验器先检查目标和值种类，分发器再写入。
/// A serialized binding assignment from an expression to a runtime-owned target surface.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct UiBindingTargetAssignment {
    /// 目标种类、可选名称和缺值策略；具体组件端点由 binding compiler 解析并验证。
    pub target: UiBindingTarget,
    /// 尚未解析的表达式源文本；binding compiler 用 `UiBindingExpression::parse` 解析并验证其预算与引用。
    pub expression: String,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
/// 可序列化的绑定写入目标声明；`name` 只用于 Prop、Class、ActionPayload，Visibility/Enabled 不带名称。
/// 便利构造函数提供 Required 缺值策略；公开字段或反序列化仍可构造不匹配的 kind/name 组合，binding validation/compiler 在生成端点前负责校验。
pub struct UiBindingTarget {
    pub kind: UiBindingTargetKind,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    /// 具名目标的字段或属性名；其含义由 `kind` 决定。
    pub name: Option<String>,
    #[serde(
        default,
        skip_serializing_if = "UiBindingMissingValuePolicy::is_required"
    )]
    /// 表达式没有结果时，由分发端解释的策略。
    pub missing_policy: UiBindingMissingValuePolicy,
}

impl UiBindingTarget {
    /// 构造 Widget 描述符属性写入目标；此方法只记录名字，目标存在性和值种类由 binding compiler 按 Widget contract 校验。
    ///
    /// # 参数
    /// `name` 是目标 Widget contract 中的属性名。
    ///
    /// # 返回值
    /// 返回 `Prop` 目标，初始缺值策略为 Required；compiler 后续将其解析为运行时 endpoint。
    pub fn prop(name: impl Into<String>) -> Self {
        Self {
            kind: UiBindingTargetKind::Prop,
            name: Some(name.into()),
            missing_policy: UiBindingMissingValuePolicy::Required,
        }
    }

    /// 构造对节点 class 集合应用表达式结果的具名目标；类名由调用方提供，compiler 负责目标与结果种类校验。
    ///
    /// # 参数
    /// `name` 是被此绑定增删的 class 名称。
    ///
    /// # 返回值
    /// 返回 `Class` 目标，初始缺值策略为 Required。
    pub fn class(name: impl Into<String>) -> Self {
        Self {
            kind: UiBindingTargetKind::Class,
            name: Some(name.into()),
            missing_policy: UiBindingMissingValuePolicy::Required,
        }
    }

    /// 构造当前节点的可见性目标；端点不带 `name`，表达式结果应满足布尔目标契约。
    pub fn visibility() -> Self {
        Self {
            kind: UiBindingTargetKind::Visibility,
            name: None,
            missing_policy: UiBindingMissingValuePolicy::Required,
        }
    }

    /// 构造当前节点的启用状态目标；端点不带 `name`，表达式结果应满足布尔目标契约。
    pub fn enabled() -> Self {
        Self {
            kind: UiBindingTargetKind::Enabled,
            name: None,
            missing_policy: UiBindingMissingValuePolicy::Required,
        }
    }

    /// 构造动作 payload 中的具名字段目标；字段和值种类需与动作 schema 相符，最终由事件分发器写入 payload。
    ///
    /// # 参数
    /// `name` 是动作 payload schema 中声明的字段名。
    ///
    /// # 返回值
    /// 返回 `ActionPayload` 目标，初始缺值策略为 Required。
    pub fn action_payload(name: impl Into<String>) -> Self {
        Self {
            kind: UiBindingTargetKind::ActionPayload,
            name: Some(name.into()),
            missing_policy: UiBindingMissingValuePolicy::Required,
        }
    }

    /// 设置表达式求值缺失时的行为；该声明由绑定求值器解析，调用端仍负责应用写入或事件路径。
    pub fn with_missing_policy(mut self, policy: UiBindingMissingValuePolicy) -> Self {
        self.missing_policy = policy;
        self
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
/// 编译器与分发器共同识别的目标类别，决定 name 的用途和最终写入端点。
pub enum UiBindingTargetKind {
    /// 写入 Widget contract 中的具名属性。
    Prop,
    /// 将布尔结果应用于节点 class 集合。
    Class,
    /// 更新节点可见性布尔值。
    Visibility,
    /// 更新节点启用状态布尔值。
    Enabled,
    /// 写入动作 payload 中的具名字段。
    ActionPayload,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
/// 编译或静态校验用目标和值种类约束；它描述契约，不负责在运行时写值。
pub struct UiBindingTargetSchema {
    /// 本次静态契约检查对应的目标声明。
    pub target: UiBindingTarget,
    /// 目标允许的表达式结果种类。
    pub value_kind: UiValueKind,
}
