// 静态文件还声明 editor/native 模块；本对照只覆盖运行时描述符提供的 runtime 模块。
use super::super::super::StaticModule;

pub(super) fn runtime_modules_from_static_modules(modules: Vec<StaticModule>) -> Vec<StaticModule> {
    modules.into_iter().filter(is_runtime_module).collect()
}

fn is_runtime_module(module: &StaticModule) -> bool {
    module.1 == zircon_runtime::plugin::PluginModuleKind::Runtime
}
