use super::super::weak::CoreWeak;

/// 模块生命周期回调的调用上下文；模块名标识本次转换的所有者。
///
/// `core` 是弱句柄，回调可临时升级访问服务，但不能靠保存上下文延长 Runtime 生命周期。
#[derive(Clone, Debug)]
pub struct ModuleContext {
    pub module_name: String,
    pub core: CoreWeak,
}
