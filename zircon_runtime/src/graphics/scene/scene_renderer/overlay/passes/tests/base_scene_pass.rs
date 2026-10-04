use super::*;

#[test]
fn wire_only_skips_only_when_color_and_depth_preserve_contents() {
    assert!(wire_only_load_store_can_skip(
        DisplayMode::WireOnly,
        RenderGraphAttachmentOps::load_store(),
        RenderGraphAttachmentOps::load_store(),
    ));
    assert!(!wire_only_load_store_can_skip(
        DisplayMode::WireOnly,
        RenderGraphAttachmentOps::clear_store(),
        RenderGraphAttachmentOps::load_store(),
    ));
    assert!(!wire_only_load_store_can_skip(
        DisplayMode::WireOnly,
        RenderGraphAttachmentOps::load_store(),
        RenderGraphAttachmentOps::clear_store(),
    ));
    assert!(!wire_only_load_store_can_skip(
        DisplayMode::Shaded,
        RenderGraphAttachmentOps::load_store(),
        RenderGraphAttachmentOps::load_store(),
    ));
}

#[test]
fn wire_only_guards_precede_binding_and_sprite_preparation() {
    let source = include_str!("../base_scene_pass.rs");
    let implementation = source
        .split("#[cfg(test)]")
        .next()
        .expect("base scene implementation");
    let opaque = implementation
        .split("pub(crate) fn record_commands_with_attachment_ops")
        .nth(1)
        .expect("opaque base scene function")
        .split("pub(crate) fn record_transparent_mixed_with_attachment_ops")
        .next()
        .expect("opaque base scene body");
    let transparent = implementation
        .split("pub(crate) fn record_transparent_mixed_with_attachment_ops")
        .nth(1)
        .expect("transparent base scene function");

    assert!(
        opaque.find("wire_only_load_store_can_skip").unwrap()
            < opaque.find("create_forward_shading_bind_group").unwrap()
    );
    assert!(
        transparent.find("wire_only_load_store_can_skip").unwrap()
            < transparent
                .find("build_transparent_submission_order")
                .unwrap()
    );
}

#[test]
fn async_base_pipeline_placeholder_skips_draws_without_retaining_stale_state() {
    let source = include_str!("../base_scene_pass.rs");
    let implementation = source
        .split("#[cfg(test)]")
        .next()
        .expect("base scene implementation");

    assert_eq!(
        implementation
            // BUG: [CR-R02-runtime_wave12_graphics_renderer_overlay-0001] 运行此测试时首个计数断言必失败：生产前缀已改用 admission 分支，旧占位绑定文本出现 0 次而断言要求 2 次；本函数已挂载为单元测试。
            .matches("let Some(pipeline) = mesh_pipelines")
            .count(),
        2,
        "opaque and transparent Base passes should explicitly consume a pending pipeline"
    );
    assert_eq!(
        implementation.matches("return false;").count(),
        2,
        "a pending Base pipeline must skip both opaque and transparent draws"
    );
    assert_eq!(
        implementation
            .matches("replayer.invalidate_state_after_external_pipeline();")
            .count(),
        3,
        "both pending-pipeline branches must invalidate replay state before the next command"
    );
    assert!(
        !implementation.contains("base mesh command must resolve a cache-backed pipeline variant"),
        "a pending async Base pipeline must not panic the frame path"
    );
}

#[test]
fn opaque_environment_only_draws_do_not_eagerly_create_or_bind_forward_receivers() {
    let source = include_str!("../base_scene_pass.rs");
    let implementation = source
        .split("#[cfg(test)]")
        .next()
        .expect("base scene implementation");
    let opaque = implementation
        .split("pub(crate) fn record_commands_with_attachment_ops")
        .nth(1)
        .expect("opaque base scene function")
        .split("pub(crate) fn record_transparent_mixed_with_attachment_ops")
        .next()
        .expect("opaque base scene body");

    assert!(
        opaque.contains("base_pipeline_requires_forward_receiver"),
        "the opaque pass must distinguish the EnvironmentOnly layout before binding group 1"
    );
    assert!(
        opaque.contains("mesh_draw_commands.clone().any"),
        "the opaque pass must determine generic ABI use before beginning the render pass"
    );
    assert!(
        opaque.contains("needs_forward_receiver && provided_forward_receiver_bind_group.is_none()"),
        "viewport recording must allocate a receiver only for a generic Base command"
    );
    assert!(
        opaque.contains(
            "provided_forward_receiver_bind_group.or(owned_forward_receiver_bind_group.as_ref())"
        ),
        "offscreen recording must be able to reuse a caller-owned receiver across passes"
    );
    assert!(
        !opaque.contains("pass.set_bind_group(1, &forward_shadow_receiver_bind_group, &[]);"),
        "the opaque pass must not bind group 1 before it knows which Base layout is active"
    );
}

#[test]
fn transparent_sprites_use_submission_indexed_preparation() {
    let source = include_str!("../base_scene_pass.rs");
    let implementation = source
        .split("#[cfg(test)]")
        .next()
        .expect("base scene implementation");
    let transparent = implementation
        .split("pub(crate) fn record_transparent_mixed_with_attachment_ops")
        .nth(1)
        .expect("transparent base scene function")
        .split("fn wire_only_load_store_can_skip")
        .next()
        .expect("transparent base scene body");

    assert!(
        transparent.contains("transparent_sprites\n                        .get(sprite_index)"),
        "transparent sprite replay should resolve the extracted sprite directly by index"
    );
    assert!(
        !transparent.contains("transparent_sprites.iter().find"),
        "transparent sprite replay must not perform a linear lookup for every submission"
    );
    assert!(
        implementation.contains("draws.resize_with(frame.sprites().len(), || None);"),
        "sprite preparation should reserve stable slots for extracted sprite indices"
    );
}
