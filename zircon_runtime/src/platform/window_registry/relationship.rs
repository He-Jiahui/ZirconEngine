//! 窗口父子图的关系标签和值载体，供注册表建立、查询和解除关系。
//! 关系只记录代际限定身份；图校验与关闭顺序由外层注册表负责。

use crate::core::framework::window::WindowId;

/// Ownership semantics for one window's relationship to its parent.
///
/// Platform backends may map these to native transient/modal/owner behavior,
/// but the registry owns the generation-qualified graph and destruction
/// ordering independent of any backend handle.
/// 宿主可据此区分原生关系意图；本枚举不直接调用平台后端或触发销毁。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum WindowParentKind {
    /// 临时从属关系标签；具体宿主映射由平台实现决定。
    Transient,
    /// 模态关系标签；仅存储此值不会创建模态窗口或改变输入路由。
    Modal,
    /// 所有者关闭关联标签；注册表的子树关闭遍历不按此标签筛选。
    OwnerShutdown,
}

/// 一个子槽位保存的父边；身份和值按复制语义保存，不借用父槽位。
/// 构造此值不证明父窗口仍存活、同属注册表或不存在环。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct WindowParentRelation {
    /// 父窗口的注册表、槽号和代际身份；有效性由注册表查询校验。
    pub(super) window: WindowId,
    /// 原生关系的意图标签，不承载平台资源。
    pub(super) kind: WindowParentKind,
}

/// 只负责封装父边；关联图的修改仍经注册表事务完成。
impl WindowParentRelation {
    /// 按值保存父身份 `window` 与关系标签 `kind`，返回不含借用的关系值。
    /// 不校验存活或环路；`WindowRegistry::set_parent` 在调用前完成这两项检查，
    /// 并负责更新双向关系，`clear_parent` 和销毁流程负责解除关系。
    pub(super) const fn new(window: WindowId, kind: WindowParentKind) -> Self {
        Self { window, kind }
    }
}
