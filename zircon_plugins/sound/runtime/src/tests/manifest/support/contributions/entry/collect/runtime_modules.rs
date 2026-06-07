use super::super::super::StaticModule;

pub(super) fn runtime_modules_from_static_modules(modules: Vec<StaticModule>) -> Vec<StaticModule> {
    modules.into_iter().filter(is_runtime_module).collect()
}

fn is_runtime_module(module: &StaticModule) -> bool {
    module.1 == zircon_runtime::plugin::PluginModuleKind::Runtime
}
