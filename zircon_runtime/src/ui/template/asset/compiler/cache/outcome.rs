use crate::ui::template::UiCompiledDocument;
use zircon_runtime_interface::ui::template::UiInvalidationReport;

/// 缓存编译同时返回可实例化结果与失效解释；命中表示内容可复用，并不表示已经安装到某个运行时 surface。
#[derive(Clone, Debug, PartialEq)]
pub struct UiCompileCacheOutcome {
    pub compiled: UiCompiledDocument,
    pub cache_hit: bool,
    /// 命中时为空报告；未命中时相对该资产上次快照解释变化，供重载调用方决定后续更新范围。
    pub invalidation_report: UiInvalidationReport,
}
