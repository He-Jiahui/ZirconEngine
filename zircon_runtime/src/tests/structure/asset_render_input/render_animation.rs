use super::*;

// TODO: [CR-RUNTIME-TESTS-STRUCT-0017] 渲染框架锁恢复守卫首先失败于旧导入写法；导入形式不足以证明恢复行为，需定位现行框架状态访问器及中毒恢复测试后修订。
#[test]
fn runtime_15_wgpu_render_framework_lock_poison_recovery_guard_covers_wgpu_framework() {
    let wgpu_framework =
        read_runtime_src("graphics/wgpu_render_framework/wgpu_render_framework.rs");
    let framework_tests =
        read_runtime_src("graphics/runtime/framework/tests/wgpu_render_framework.rs");

    assert_contains_all(
        "WGPU render framework poison recovery helpers",
        &wgpu_framework,
        &[
            "pub(in crate::graphics::runtime::render_framework) fn lock_operation(",
            "pub(in crate::graphics::runtime::render_framework) fn lock_state(",
            ".unwrap_or_else(|poisoned| poisoned.into_inner())",
            "#[path = \"../runtime/framework/tests/wgpu_render_framework.rs\"]",
        ],
    );
    assert_contains_all(
        "WGPU render framework recovery regression",
        &framework_tests,
        &[
            "fn wgpu_render_framework_accessors_recover_poisoned_locks()",
            "framework.core.operation_lock.lock().unwrap()",
            "framework.core.state.lock().unwrap()",
            "drop(framework.lock_operation());",
            "assert!(framework.lock_state().viewports.is_empty());",
        ],
    );
    assert_no_direct_lock_unwrap_in_production("WGPU render framework", &wgpu_framework);
    assert!(
        !production_section(&wgpu_framework).contains("lock poisoned"),
        "WGPU render framework production paths should recover poisoned locks instead of panicking"
    );
}

// TODO: [CR-RUNTIME-TESTS-STRUCT-0016] 动画播放设置写入未匹配旧锁访问表达式；尚未证明恢复策略失效，需沿设置更新入口检查现行锁封装和中毒写入测试。
#[test]
fn runtime_15_animation_manager_lock_poison_recovery_guard_covers_playback_settings() {
    let animation_manager = read_runtime_src("animation/manager/mod.rs");
    let animation_manager_tests = read_runtime_src("animation/manager/tests/cases.rs");

    assert_contains_all(
        "DefaultAnimationManager playback settings poison recovery",
        &animation_manager,
        &[
            "use std::sync::{Arc, Mutex, MutexGuard};",
            "fn lock_playback_settings(&self) -> MutexGuard<'_, AnimationPlaybackSettings>",
            ".unwrap_or_else(|poisoned| poisoned.into_inner())",
            "*self.lock_playback_settings() = playback_settings;",
            "self.lock_playback_settings().clone()",
            "#[path = \"tests/cases.rs\"]",
        ],
    );
    assert_contains_all(
        "animation playback settings recovery regression",
        &animation_manager_tests,
        &[
            "fn animation_manager_playback_settings_recover_poisoned_lock()",
            "let _guard = manager.lock_playback_settings();",
            ".store_playback_settings(playback_settings.clone())",
            "assert_eq!(manager.playback_settings(), playback_settings);",
        ],
    );
    assert_no_direct_lock_unwrap_in_production("animation manager", &animation_manager);
    assert!(
        !production_section(&animation_manager).contains("animation playback mutex poisoned"),
        "DefaultAnimationManager production paths should recover poisoned playback settings locks instead of panicking"
    );
}
