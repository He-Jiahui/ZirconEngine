use std::path::PathBuf;

use super::super::weak::CoreWeak;

/// 插件工厂收到的运行时访问权与可选包路径上下文。
///
/// Core 注册表实例化时只提供插件名与弱句柄；VM 包加载流程可补充三个根目录。
/// 插件须处理 Runtime 已卸载或路径不可用的情况。
#[derive(Clone, Debug)]
pub struct PluginContext {
    /// 当前插件上下文的名称；Core 服务工厂入口传入完整服务注册名。
    pub plugin_name: String,
    /// 可克隆的弱回访入口；保存它不会保活 Runtime。
    pub core: CoreWeak,
    /// 可选包根目录；Core 服务解析入口不提供，VM 加载器可按包来源补充。
    pub package_root: Option<PathBuf>,
    /// 可选源码查找根，由 VM 包来源推导；不能从包根存在推断本字段一定存在。
    pub source_root: Option<PathBuf>,
    /// 可选数据根；VM 路径推导会在包根下拼接 data，本字段本身不创建目录。
    pub data_root: Option<PathBuf>,
}
