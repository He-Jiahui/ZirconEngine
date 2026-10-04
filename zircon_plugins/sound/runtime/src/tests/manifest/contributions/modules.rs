// 只比较静态清单声明的 runtime 模块；包清单还会由运行时入口附加 native 发布模块。
use super::super::support::{static_sound_contributions, STATIC_SOUND_PLUGIN_MANIFEST};

#[test]
fn static_plugin_manifest_keeps_runtime_modules_in_sync() {
    let static_manifest = static_sound_contributions(STATIC_SOUND_PLUGIN_MANIFEST);
    let runtime_manifest = crate::package_manifest();
    let mut runtime_modules = runtime_manifest
        .modules
        .iter()
        .filter(|module| module.kind == zircon_runtime::plugin::PluginModuleKind::Runtime)
        .map(|module| {
            (
                module.name.clone(),
                module.kind,
                module.crate_name.clone(),
                module.target_modes.clone(),
                module.capabilities.clone(),
            )
        })
        .collect::<Vec<_>>();

    runtime_modules.sort_unstable_by_key(|module| module.0.clone());

    assert_eq!(static_manifest.modules, runtime_modules);
}
