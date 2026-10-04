// 静态依赖声明必须与 package_manifest 的运行时依赖一致，避免导出规划和运行时装载看到不同要求。
use super::super::support::{static_sound_contributions, STATIC_SOUND_PLUGIN_MANIFEST};

#[test]
fn static_plugin_manifest_keeps_runtime_dependencies_in_sync() {
    let static_manifest = static_sound_contributions(STATIC_SOUND_PLUGIN_MANIFEST);
    let runtime_manifest = crate::package_manifest();
    let mut runtime_dependencies = runtime_manifest
        .dependencies
        .iter()
        .map(|dependency| {
            (
                dependency.id.clone(),
                dependency.required,
                dependency.capability.clone(),
            )
        })
        .collect::<Vec<_>>();

    runtime_dependencies.sort_unstable();

    assert_eq!(static_manifest.dependencies, runtime_dependencies);
}
