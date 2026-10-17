//! 事件调用、控制请求和订阅通知的可序列化数据。
//!
//! 消息拥有路径、参数与结果；路由查找、远程权限、属性写入及通知发送由消费方执行。
//! 编号只标识管理器中的项目，不拥有回调或接收器，也不保证项目仍然存在。

use serde::{Deserialize, Serialize};
use serde_json::Value;
use thiserror::Error;

use crate::ui::binding::{UiBindingValue, UiEventBinding};

use super::{
    UiNodeDescriptor, UiNodePath, UiPropertyDescriptor, UiReflectionDiff, UiReflectionSnapshot,
};

/// 为本模块的路由和订阅生成同形编号，不承担编号分配。
macro_rules! define_u64_id {
    ($name:ident) => {
        /// 原样保存的数值编号；默认零值和任意输入均不会在本类型内校验。
        ///
        /// 有效性、唯一性及释放后的查找结果由持有该编号的管理器决定。
        #[derive(
            Clone,
            Copy,
            Debug,
            Default,
            PartialEq,
            Eq,
            PartialOrd,
            Ord,
            Hash,
            Serialize,
            Deserialize,
        )]
        pub struct $name(pub u64);

        impl $name {
            /// 保存 `value` 并返回编号；不查表、分配或注册管理器项目。
            pub const fn new(value: u64) -> Self {
                Self(value)
            }
        }
    };
}

define_u64_id!(UiRouteId);
define_u64_id!(UiSubscriptionId);

/// 调用来源的拥有记录，用于区分直接路由、绑定和带节点路径的动作入口。
///
/// 该标记不执行权限检查；动作路径与名称也不延长节点或路由的有效期。
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum UiInvocationSource {
    Route,
    Binding,
    Action {
        node_path: UiNodePath,
        action_id: String,
    },
}

/// 交给路由处理器的一次调用上下文。
///
/// # 约束与生命周期
/// 字符串、绑定和参数均由上下文拥有，处理器可按其自身约定继续持有。
/// `binding` 是绑定描述，`arguments` 是独立参数槽；本类型不合并默认参数或校验参数模式。
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct UiInvocationContext {
    pub route_id: UiRouteId,
    pub binding: UiEventBinding,
    pub arguments: Vec<UiBindingValue>,
    pub source: UiInvocationSource,
}

/// 控制面查询、权限、路由和处理器失败的可序列化诊断。
///
/// 各变体保存发现错误时的定位数据；声明和反序列化不会自行查询项目或触发错误检查。
/// 展示文本由各变体的错误格式定义，实际失败条件由调用方产生。
#[derive(Clone, Debug, Error, PartialEq, Eq, Serialize, Deserialize)]
pub enum UiInvocationError {
    #[error("unknown binding {0}")]
    UnknownBinding(String),
    #[error("unknown route {0:?}")]
    UnknownRoute(UiRouteId),
    #[error("unknown tree {0}")]
    UnknownTree(String),
    #[error("unknown node {0}")]
    UnknownNode(String),
    #[error("unknown property {property_name} on {node_path}")]
    UnknownProperty {
        node_path: String,
        property_name: String,
    },
    #[error("property {property_name} on {node_path} is not writable")]
    PropertyNotWritable {
        node_path: String,
        property_name: String,
    },
    #[error("unknown action {action_id} on {node_path}")]
    UnknownAction {
        node_path: String,
        action_id: String,
    },
    #[error("action {action_id} on {node_path} is not callable remotely")]
    ActionNotCallable {
        node_path: String,
        action_id: String,
    },
    #[error("action {action_id} on {node_path} is missing a registered route")]
    ActionMissingRoute {
        node_path: String,
        action_id: String,
    },
    #[error("subscription {0:?} is missing")]
    MissingSubscription(UiSubscriptionId),
    #[error("handler failed: {0}")]
    HandlerFailed(String),
}

/// 调用或控制操作的拥有结果，可携带已知的路由、绑定及成功值或错误。
///
/// # 约束
/// 两个工厂分别建立成功值与错误互斥的常见形状；公有字段和反序列化不额外强制该关系。
/// 尚未解析路由或绑定的失败可将相应字段留空。
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct UiInvocationResult {
    pub route_id: Option<UiRouteId>,
    pub binding: Option<UiEventBinding>,
    pub value: Option<Value>,
    pub error: Option<UiInvocationError>,
}

impl UiInvocationResult {
    /// 以已知路由、绑定和成功值构造结果，错误槽留空。
    ///
    /// 三个参数均移入返回值；此处不调用处理器或发送通知。
    pub fn success(route_id: UiRouteId, binding: UiEventBinding, value: Value) -> Self {
        Self {
            route_id: Some(route_id),
            binding: Some(binding),
            value: Some(value),
            error: None,
        }
    }

    /// 以可选定位信息和错误构造失败结果，成功值槽留空。
    ///
    /// `route_id`、`binding` 可表达尚未解析出的信息；错误与绑定移入返回值。
    pub fn failure(
        route_id: Option<UiRouteId>,
        binding: Option<UiEventBinding>,
        error: UiInvocationError,
    ) -> Self {
        Self {
            route_id,
            binding,
            value: None,
            error: Some(error),
        }
    }
}

/// 供订阅通道传递的拥有载荷；数据本身不持有通道或保证接收器仍存活。
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum UiNotification {
    ReflectionDiff(UiReflectionDiff),
    Invocation(UiInvocationResult),
}

/// 控制面的序列化请求：Runtime 的事件管理器处理路由调用与反射查询，Editor 服务沿用同一请求边界。
///
/// # 使用时机
/// 由控制服务交给实际管理器逐变体处理；请求拥有参数，不借用发送方的临时输入。
/// 查询、属性设置和动作调用的检查及错误应答由执行方产生，本声明不执行这些操作。
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum UiControlRequest {
    InvokeBinding {
        binding: UiEventBinding,
    },
    InvokeRoute {
        route_id: UiRouteId,
        arguments: Vec<UiBindingValue>,
    },
    QueryTree {
        tree_id: super::UiTreeId,
    },
    QueryNode {
        node_path: UiNodePath,
    },
    QueryProperty {
        node_path: UiNodePath,
        property_name: String,
    },
    SetProperty {
        node_path: UiNodePath,
        property_name: String,
        value: Value,
    },
    CallAction {
        node_path: UiNodePath,
        action_id: String,
        arguments: Vec<UiBindingValue>,
    },
    /// 请求订阅编号；通知接收器的交付和保活仍需由实际控制服务负责。
    SubscribeDiffs,
    Unsubscribe {
        subscription_id: UiSubscriptionId,
    },
}

/// 控制服务的拥有应答：查询项可缺失，操作可返回确认或调用结果。
///
/// `Subscription` 只携带编号，不包含通知接收器；读取该值不会建立接收循环。
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum UiControlResponse {
    Invocation(UiInvocationResult),
    Tree(Option<UiReflectionSnapshot>),
    Node(Option<UiNodeDescriptor>),
    Property(Option<UiPropertyDescriptor>),
    Subscription(UiSubscriptionId),
    Ack,
}

/// 调用侧沿用控制请求的类型别名，不引入额外序列化层。
pub type UiInvocationRequest = UiControlRequest;
/// 调用侧沿用控制应答的类型别名，不改变应答字段或所有权。
pub type UiInvocationResponse = UiControlResponse;
