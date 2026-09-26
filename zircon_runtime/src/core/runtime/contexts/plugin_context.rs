use std::path::PathBuf;

use super::super::weak::CoreWeak;

/// 插件工厂收到的运行时访问权与可选包路径上下文。
///
/// Core 注册表实例化时只提供插件名与弱句柄；VM 包加载流程可补充三个根目录。
/// 插件须处理 Runtime 已卸载或路径不可用的情况。
#[derive(Clone, Debug)]
pub struct PluginContext {
    pub plugin_name: String,
    pub core: CoreWeak,
    pub package_root: Option<PathBuf>,
    pub source_root: Option<PathBuf>,
    pub data_root: Option<PathBuf>,
}
