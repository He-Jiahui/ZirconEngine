#[cfg(debug_assertions)]
use std::sync::atomic::{AtomicU64, Ordering};

/// 弱桥接调用的诊断快照；计数仅在调试构建中维护，发布构建返回零。
///
/// # 职责与使用时机
/// 保存冻结表单个 entry 的入口事件观测值，供诊断报告读取。
/// 字段记录准入与拒绝事件，不作为业务结果或后续调用许可。
///
/// # 时间与构造边界
/// 当前内部记录器生成此类型时分别读取两个原子字段，因此两值不保证来自同一时刻。
/// 未启用 `debug_assertions` 时内部记录器返回零；该类型也可通过默认构造取得全零值。
/// 读取后的值不自动跟随后续事件变化，不包含采样时间或表身份。
///
/// # 参考
/// [`Self::enabled_calls`]、[`Self::not_enabled_calls`]、[`super::BridgeInvocationTable`]。
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct BridgeDiagnosticsSnapshot {
    /// 已准入入口事件的累计观测次数。
    ///
    /// # 统计范围
    /// 弱桥接、VM 与原生入口在业务回调前递增，因此回调失败仍计数。
    /// 一次成功的 `pin` 计数一次，经 guard 后续解引用或调用 provider 不再递增。
    /// 两字段分别采样，成对值不保证来自同一时刻；内部记录器仅在启用
    /// `debug_assertions` 时维护计数，未启用时返回零。
    ///
    /// # 参考
    /// [`super::BridgeInvocationTable::record_enabled_call`]、[`Self::not_enabled_calls`]。
    pub enabled_calls: u64,
    /// 已定位槽位的调用在准入阶段被拒绝的累计观测次数。
    ///
    /// # 统计范围
    /// 接口不可调用或原生库回调租约被拒绝时记录。缺失 entry 没有计数归属；
    /// 原生入口到达方法查找失败分支时不为该分支递增，业务回调返回错误也不计入。
    /// 两字段分别采样，成对值不保证来自同一时刻；内部记录器仅在启用
    /// `debug_assertions` 时维护计数，未启用时返回零。
    ///
    /// # 参考
    /// [`super::BridgeInvocationTable::record_not_enabled_call`]、[`Self::enabled_calls`]。
    pub not_enabled_calls: u64,
}

#[derive(Debug, Default)]
pub(crate) struct BridgeDiagnostics {
    #[cfg(debug_assertions)]
    enabled_calls: AtomicU64,
    #[cfg(debug_assertions)]
    not_enabled_calls: AtomicU64,
}

impl BridgeDiagnostics {
    pub(crate) fn record_enabled_call(&self) {
        #[cfg(debug_assertions)]
        self.enabled_calls.fetch_add(1, Ordering::Relaxed);
    }

    pub(crate) fn record_not_enabled_call(&self) {
        #[cfg(debug_assertions)]
        self.not_enabled_calls.fetch_add(1, Ordering::Relaxed);
    }

    /// 读取两项入口诊断计数，生成供表报告使用的值快照。
    ///
    /// # 返回
    /// 返回 [`BridgeDiagnosticsSnapshot`]；启用 `debug_assertions` 的构建分别读取两个原子字段，未启用时两项都为零。
    ///
    /// # 采样边界
    /// 两个 `Relaxed` 读取彼此独立，不保证两项来自同一时刻。计数只用于诊断，
    /// 不参与 provider 与代际的发布同步，也不能作为调用准入许可。
    /// 返回后该值不自动跟随后续计数变化。
    ///
    /// # 参考
    /// [`Self::record_enabled_call`]、[`Self::record_not_enabled_call`]、[`BridgeDiagnosticsSnapshot`]。
    pub(crate) fn snapshot(&self) -> BridgeDiagnosticsSnapshot {
        BridgeDiagnosticsSnapshot {
            enabled_calls: enabled_calls(self),
            not_enabled_calls: not_enabled_calls(self),
        }
    }
}

#[cfg(debug_assertions)]
fn enabled_calls(diagnostics: &BridgeDiagnostics) -> u64 {
    diagnostics.enabled_calls.load(Ordering::Relaxed)
}

#[cfg(not(debug_assertions))]
fn enabled_calls(_: &BridgeDiagnostics) -> u64 {
    0
}

#[cfg(debug_assertions)]
fn not_enabled_calls(diagnostics: &BridgeDiagnostics) -> u64 {
    diagnostics.not_enabled_calls.load(Ordering::Relaxed)
}

#[cfg(not(debug_assertions))]
fn not_enabled_calls(_: &BridgeDiagnostics) -> u64 {
    0
}
