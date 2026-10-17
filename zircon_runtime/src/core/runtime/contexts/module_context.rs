use super::super::weak::CoreWeak;

/// 模块生命周期回调的调用上下文；模块名标识本次转换的所有者。
///
/// `core` 是弱句柄，回调可临时升级访问服务，但不能靠保存上下文延长 Runtime 生命周期。
#[derive(Clone, Debug)]
pub struct ModuleContext {
    /// 本次生命周期转换的模块名；由调度入口复制，不代表另一次转换的状态。
    pub module_name: String,
    /// 非拥有式回访入口；需要访问 Runtime 时显式升级并处理升级失败。
    pub core: CoreWeak,
}
