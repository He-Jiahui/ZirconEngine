use super::*;

#[test]
fn runtime_15_wgpu_render_framework_lock_poison_recovery_guard_covers_wgpu_framework() {
    let wgpu_framework = read_runtime_src(
        "graphics/runtime/render_framework/wgpu_render_framework/wgpu_render_framework.rs",
    );
    let runtime_15_plan =
        read_repo("docs/plans/zircon_runtime/runtime/15-code-structure-and-module-conventions.md");
    let runtime_index = read_repo("docs/plans/zircon_runtime/runtime/index.md");
    let review_findings = read_repo("docs/plans/engine-code-review-findings-2026-06.md");
    let structure_convention = read_repo("docs/plans/engine-code-structure-convention.md");
    let module_doc = read_repo("docs/zircon_runtime/structure/module-convention.md");
    let render_product_doc = read_repo("docs/zircon_runtime/graphics/render-product-submit.md");

    assert_contains_all(
        "WGPU render framework poison recovery helpers",
        &wgpu_framework,
        &[
            "use std::sync::{Mutex, MutexGuard};",
            "pub(in crate::graphics::runtime::render_framework) fn lock_operation(",
            "pub(in crate::graphics::runtime::render_framework) fn lock_state(",
            ".unwrap_or_else(|poisoned| poisoned.into_inner())",
            "wgpu_render_framework_accessors_recover_poisoned_locks",
        ],
    );
    assert_no_direct_lock_unwrap_in_production("WGPU render framework", &wgpu_framework);
    assert!(
        !production_section(&wgpu_framework).contains("lock poisoned"),
        "WGPU render framework production paths should recover poisoned locks instead of panicking"
    );
}

#[test]
fn runtime_15_animation_manager_lock_poison_recovery_guard_covers_playback_settings() {
    let animation_manager = read_runtime_src("animation/manager/mod.rs");
    let runtime_15_plan =
        read_repo("docs/plans/zircon_runtime/runtime/15-code-structure-and-module-conventions.md");
    let runtime_index = read_repo("docs/plans/zircon_runtime/runtime/index.md");
    let review_findings = read_repo("docs/plans/engine-code-review-findings-2026-06.md");
    let structure_convention = read_repo("docs/plans/engine-code-structure-convention.md");
    let module_doc = read_repo("docs/zircon_runtime/structure/module-convention.md");
    let animation_doc = read_repo("docs/zircon_runtime/animation/runtime.md");

    assert_contains_all(
        "DefaultAnimationManager playback settings poison recovery",
        &animation_manager,
        &[
            "use std::sync::{Arc, Mutex, MutexGuard};",
            "fn lock_playback_settings(&self) -> MutexGuard<'_, AnimationPlaybackSettings>",
            ".unwrap_or_else(|poisoned| poisoned.into_inner())",
            "*self.lock_playback_settings() = playback_settings.clone();",
            "self.lock_playback_settings().clone()",
            "animation_manager_playback_settings_recover_poisoned_lock",
        ],
    );
    assert_no_direct_lock_unwrap_in_production("animation manager", &animation_manager);
    assert!(
        !production_section(&animation_manager).contains("animation playback mutex poisoned"),
        "DefaultAnimationManager production paths should recover poisoned playback settings locks instead of panicking"
    );
}
