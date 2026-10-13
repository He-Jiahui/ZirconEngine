//! Runtime 世界事实订阅的传输类型；具体匹配索引和失效规则由 Runtime 订阅表持有。
//!
//! Editor 将 WatchRegistration 交给当前 Runtime 会话，得到只能在该会话内解释的
//! WatchToken。撤销 token 后 Runtime 不再为它保留订阅；Editor 自己的视图身份由 Editor
//! 绑定表维护，不随这些 DTO 发送。
//!
//! # 参考
//!
//! Runtime 分配和撤销 token 的入口见 zircon_runtime/src/dynamic_api/session/world_sync.rs；
//! 按事实类别匹配订阅的实现见 zircon_runtime/src/scene/inspection/subscription.rs。

use serde::{Deserialize, Serialize};

use crate::resource::ResourceId;

use super::EntityId;

/// Editor 订阅的是 Runtime 事实类别；视图实例身份留在 Editor 的绑定表内。
/// A runtime-owned fact family that a view or tool can subscribe to.
///
/// Editor 按事实类别注册订阅，Runtime 的 SubscriptionTable 据此把变化映射为 dirty token。
/// 此键不含 Editor 视图标识；Subtree 和 ComponentType 的匹配细节由 Runtime 订阅表实现。
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(
    tag = "kind",
    content = "data",
    rename_all = "snake_case",
    deny_unknown_fields
)]
pub enum WatchKey {
    /// 关注指定实体根下的层级变化；root 指被订阅子树的根实体。
    Subtree { root: EntityId },
    /// 关注完整反射类型路径对应组件的变化；type_name 是反射注册中的组件路径。
    ComponentType { type_name: String },
    /// 关注指定资源身份对应的变化；resource_id 是被订阅资源的稳定身份。
    Asset { resource_id: ResourceId },
    /// 关注 Runtime 世界结构事实。
    WorldStructure,
}

/// A transport request for one independently revocable watch.
///
/// 注册请求在 Runtime 会话内创建一个独立 token；同一键可注册多次并分别撤销。
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct WatchRegistration {
    /// Runtime 用于选择事实类别的订阅键。
    pub key: WatchKey,
}

impl WatchRegistration {
    /// 用订阅键创建一个 Runtime watch 注册请求。
    ///
    /// # 参数
    ///
    /// key 指定要观察的 Runtime 事实类别；构造器不分配 token。
    pub fn new(key: WatchKey) -> Self {
        Self { key }
    }
}

/// Token 只在签发它的 Runtime 会话内有效，Editor 须另以会话身份限定其作用域。
/// Runtime-issued opaque subscription identity; runtime never stores editor view ids.
///
/// 透明序列化为一个 u64。零值保留为无效 token；Runtime 分配器跳过零值，Editor 网关在
/// 调用撤销入口前拒绝无效 token。公开构造器仍允许调用者构造零值以表示或测试无效输入。
#[repr(transparent)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct WatchToken(u64);

impl WatchToken {
    /// 从原始数值构造 token；此函数不检查数值是否对应活动订阅。
    ///
    /// # 参数
    ///
    /// value 为 token 的原始 u64 值，零表示无效 token。
    pub const fn new(value: u64) -> Self {
        Self(value)
    }

    /// 取回 token 的原始 u64 数值。
    pub const fn value(self) -> u64 {
        self.0
    }

    /// 判断该数值是否为 Runtime 可分配的非零 token。
    ///
    /// 此检查不证明 token 属于当前 Runtime 会话或仍对应活动订阅。
    pub const fn is_valid(self) -> bool {
        self.0 != 0
    }
}
