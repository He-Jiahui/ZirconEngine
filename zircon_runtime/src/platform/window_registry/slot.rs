//! 窗口注册表内部槽位的存储；生存期由外层注册表的关闭和销毁流程推进。
//! 默认值只表示尚未占用的初始槽位；复用旧槽位时保留外层已推进的代际。

use std::num::NonZeroU32;

use crate::core::framework::window::{NativeWindowId, WindowId};
use zircon_runtime_interface::ZrRuntimeViewportHandle;

use super::relationship::WindowParentRelation;

/// 保存一代窗口的身份映射、关系边与视口身份。
///
/// 字段只拥有数值及容器，不持有窗口或表面资源的借用；资源释放由外层宿主完成。
/// 活跃性、双向映射与关系一致性均由 `WindowRegistry` 校验，本载体不自行维护。
pub(super) struct WindowRegistrySlot {
    /// 发放 `WindowId` 时使用的非零代际；销毁成功后的推进由外层负责。
    pub(super) generation: NonZeroU32,
    /// 当前占用的原生窗口身份；`None` 也可能表示代际耗尽而永久退役的槽位。
    pub(super) native_window: Option<NativeWindowId>,
    /// 已进入关闭流程时停止常规路由；此标记本身不表示外部资源已释放。
    pub(super) closing: bool,
    /// 父窗口的代际限定身份与关系类别，须与父槽位的 `children` 一致。
    pub(super) parent: Option<WindowParentRelation>,
    /// 子窗口身份列表，与各子槽位的 `parent` 构成双向关系。
    pub(super) children: Vec<WindowId>,
    /// 绑定的 ABI 视口身份；关闭时移入关闭回执，并由注册表同步撤销反向索引。
    pub(super) viewports: Vec<ZrRuntimeViewportHandle>,
}

/// 仅提供新槽位的初始存储状态，不承担旧槽位复用或外部资源清理。
impl Default for WindowRegistrySlot {
    /// 返回代际从 1 开始、无窗口、关系及视口绑定的槽位。
    /// `WindowRegistry::allocate_slot` 只在新增长度时使用它；复用走空闲列表。
    fn default() -> Self {
        Self {
            generation: NonZeroU32::MIN,
            native_window: None,
            closing: false,
            parent: None,
            children: Vec::new(),
            viewports: Vec::new(),
        }
    }
}
