use serde::{Deserialize, Serialize};

use super::InterfaceSlot;

/// Marker trait for typed plugin interfaces exported through the runtime bridge table.
///
/// Implementations must be `Send + Sync + 'static` and must not borrow from an ECS `World`.
/// World access belongs in scheduled systems so the scheduler can keep conflict information
/// authoritative.
///
/// # 职责与使用时机
/// 插件通过此标记把 Rust 接口关联到桥表注册键；导出与导入使用同一静态 ID，重复导出由插件注册表拒绝。
///
/// # 生命周期与参考
/// 当前插件表以共享所有权句柄持有 provider；ECS 访问继续遵守上面的调度约定。
/// 参见 [`Self::INTERFACE_ID`]、[`super::StrongBridge`] 和 [`BridgeInvocationTable`]。
pub trait PluginInterface: Send + Sync + 'static {
    /// Globally unique interface id such as `physics.query.v1`.
    /// 导出登记、消费方导入与槽位解析共同使用的静态身份；两侧须约定相同值。
    const INTERFACE_ID: &'static str;
}

/// 类型化桥接调用的查找与准入错误，供调用方区分缺失接口与当前不可用。
///
/// # 错误含义
/// [`Self::Absent`] 表示接口或槽位未能定位；[`Self::NotEnabled`] 表示当前无法取得
/// 可调用的 provider。当前插件表的 provider 类型下转失败也返回 `NotEnabled`，
/// 因此不能把该错误唯一解释为生命周期禁用。
///
/// # 使用边界
/// 这些状态不覆盖业务回调自身返回的所有错误，也不是原生 ABI 的状态枚举。
/// 本类型不携带表、provider 或持续有效的调用许可。
///
/// # 参考
/// [`PluginInterface`]、[`BridgeInterfaceStatus`]。
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum BridgeError {
    /// The target interface is installed but disabled or currently between generations.
    NotEnabled,
    /// The target interface is not installed in the frozen bridge table.
    Absent,
}

/// 冻结表中接口的可见状态；禁用包括代际切换期间的暂时不可调用状态。
///
/// # 状态含义
/// [`Self::Absent`] 表示当前表查找不到对应 entry；[`Self::Enabled`] 表示查询时满足
/// 当前实现的可见可调用条件；[`Self::Disabled`] 表示 entry 已存在但不满足这些条件。
/// 当前插件表在 provider 被清空后仍保留 entry，因此这时报告 `Disabled`。
///
/// # 时间与验证边界
/// 当前插件表从同一次 entry 状态读取中取得代际和 provider 存在标记。
/// 该状态不检查具体方法或 provider 的 Rust 类型，不保证之后的回调仍能准入。
///
/// # 参考
/// [`BridgeInvocationTable::interface_status_at`]、[`InterfaceSlot`]。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BridgeInterfaceStatus {
    Absent,
    Enabled,
    Disabled,
}

impl BridgeInterfaceStatus {
    // 状态由插件域发布的代际奇偶与 provider 快照共同决定，弱引用调用据此失效缓存。
    /// 将已存在 entry 的代际与 provider 存在标记换算为可见状态。
    ///
    /// # 参数
    /// - `generation`：当前 entry 发布的代际；此换算以偶数代际作为可启用条件。
    /// - `provider_installed`：同一次 entry 状态读取中是否存在 provider。
    ///
    /// # 返回
    /// 偶数代际且 provider 存在时返回 [`Self::Enabled`]；其他组合返回 [`Self::Disabled`]。
    /// 本函数处理的是已存在 entry，不产生 [`Self::Absent`]。
    ///
    /// # 调用时机与前提
    /// 调用方先确认 entry 存在，并从同一状态读取取得两个参数；当前插件表的状态快照满足此前提。
    /// 本函数只做状态换算，不发布新代际或改变 provider。
    ///
    /// # 参考
    /// [`BridgeInvocationTable::interface_status_at`]、[`Self::Enabled`]、[`Self::Disabled`]。
    pub(crate) fn from_installed_entry(generation: u32, provider_installed: bool) -> Self {
        if generation % 2 == 0 && provider_installed {
            Self::Enabled
        } else {
            Self::Disabled
        }
    }
}

/// 插件拥有者在帧边界变更接口可用性时写入生命周期报告的原因。
///
/// 此值描述请求的操作；报告中的各 entry 快照给出操作后的实际状态。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BridgeOwnerTransitionMode {
    /// 请求启用或恢复 owner 的已有槽位；可调用状态仍取决于当前 provider 与代际。
    Activate,
    /// 将已有 entry 切到禁用代际，同时保留已安装的 provider。
    Disable,
    /// 禁用已有 entry 并清空表内 provider；entry 与槽位仍保留。
    Deactivate,
    /// 替换 owner 匹配槽位的 provider；原为禁用代际的 entry 继续保持禁用。
    Reload,
}

/// Neutral read/call surface consumed by runtime domains that invoke bridge methods.
///
/// Concrete plugin registration and lifecycle ownership stays in the plugin domain. Consumers
/// depend only on this frozen-table view, so they do not need the plugin facade or manifest types.
///
/// # 职责与使用时机
/// 调用方在注册宿主方法时解析接口位置，并在进入回调时查询当时状态、记录入口事件。
/// 本接口提供表视图，不执行 provider 回调。
///
/// # 调用者与实现者的边界
/// 调用者使用同一冻结表解析得到的槽位，不能把瞬时状态当作持续的调用许可。
/// 实现者关联接口 ID、槽位、可见状态和诊断入口；注册及生命周期切换继续由插件域负责。
/// 各方法注明的插件冻结表行为描述当前实现，不要求其他实现采用相同内部原子字段或采样方式。
///
/// # 参考
/// [`InterfaceSlot`]、[`BridgeInterfaceStatus`]、[`Self::resolve_interface_slot`]、
/// [`Self::interface_status_at`]、[`super::BridgeDiagnosticsSnapshot`]。
pub trait BridgeInvocationTable: Clone + Send + Sync + 'static {
    /// 将接口 ID 解析为本冻结表内的槽位，供注册的调用入口保存位置。
    ///
    /// # 参数
    /// - `interface_id`：接口注册使用的 ID；借用只用于查找，不转移字符串所有权。
    ///
    /// # 返回
    /// `Some(slot)` 表示 ID 在表中有位置，不能证明接口当前可调用；缺失 ID 返回 `None`。
    /// 当前插件冻结表在禁用或停用时保留 entry 和槽位，因此仍能解析该 ID。
    ///
    /// # 调用时机与约束
    /// 注册宿主方法时解析一次，进入回调前使用 [`Self::interface_status_at`] 查询当时状态。
    /// 返回值仅属于本表，不作为跨表或持久化的接口身份。
    ///
    /// # 参考
    /// [`InterfaceSlot`]、[`Self::interface_status_at`]。
    fn resolve_interface_slot(&self, interface_id: &str) -> Option<InterfaceSlot>;

    /// 查询指定槽位当时的可见状态，供调用入口判断是否继续准入。
    ///
    /// # 参数
    /// - `slot`：与本表关联的 [`InterfaceSlot`]；构造槽位本身不能证明 entry 存在。
    ///
    /// # 返回
    /// 槽位不存在时返回 [`BridgeInterfaceStatus::Absent`]。当前插件冻结表只有在已存在 entry
    /// 的同一次状态读取中观察到偶数代际且 provider 存在时才返回
    /// [`BridgeInterfaceStatus::Enabled`]；其他已存在 entry 返回 [`BridgeInterfaceStatus::Disabled`]。
    ///
    /// # 调用时机与边界
    /// 在进入回调前查询。结果只描述查询时状态，不检查方法是否存在或 provider 的 Rust 类型，
    /// 也不保证之后的回调仍可准入；各调用入口继续负责自身的查找与准入检查。
    ///
    /// # 参考
    /// [`Self::resolve_interface_slot`]、[`BridgeInterfaceStatus`]。
    fn interface_status_at(&self, slot: InterfaceSlot) -> BridgeInterfaceStatus;

    /// 由调用入口为指定槽位记录一次已准入事件，供诊断快照读取。
    ///
    /// # 参数
    /// - `slot`：本表中该次准入事件归属的槽位。
    ///
    /// # 返回与作用
    /// 返回 `()`；不执行回调、校验回调结果或切换接口状态。当前插件冻结表在 entry
    /// 不存在时不记录事件；其内部计数仅在启用 `debug_assertions` 的构建维护，未启用时读到零。
    ///
    /// # 调用时机与约束
    /// 准入条件由调用入口判断。当前 VM 入口在回调前记录，因此回调随后返回错误仍保留这次计数。
    /// 此方法是统计入口，不能代替 [`Self::interface_status_at`] 的状态查询。
    ///
    /// # 参考
    /// [`Self::record_not_enabled_call`]、[`super::BridgeDiagnosticsSnapshot`]。
    fn record_enabled_call(&self, slot: InterfaceSlot);

    /// 由调用入口记录一个已定位槽位的准入拒绝事件。
    ///
    /// # 参数
    /// - `slot`：本表中被拒绝的调用所归属的槽位；缺失接口没有已定位的 entry。
    ///
    /// # 返回与作用
    /// 返回 `()`，只记录诊断事件，不改变接口状态或创建 entry。当前插件冻结表遇到不存在
    /// 的槽位时不记录；其内部计数仅在启用 `debug_assertions` 的构建维护，未启用时读到零。
    ///
    /// # 调用时机与约束
    /// 在调用入口判断不能准入时记录。当前 VM 的缺失槽位分支直接返回错误，不调用此方法；
    /// 不可调用的既有槽位分支才记录。该指标不能代表业务回调返回的所有错误。
    ///
    /// # 参考
    /// [`Self::record_enabled_call`]、[`Self::interface_status_at`]、[`super::BridgeDiagnosticsSnapshot`]。
    fn record_not_enabled_call(&self, slot: InterfaceSlot);
}
