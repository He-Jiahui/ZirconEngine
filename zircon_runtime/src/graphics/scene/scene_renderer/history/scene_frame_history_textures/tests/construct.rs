#[test]
fn history_initialization_uses_gpu_clear_passes_without_cpu_texture_payloads() {
    let source = include_str!("../construct.rs");
    let implementation = source
        .split("#[cfg(test)]")
        .next()
        .expect("history texture construction implementation");

    assert!(!implementation.contains("Vec::with_capacity("));
    assert!(!implementation.contains("write_texture("));
    assert_eq!(implementation.matches("begin_render_pass(").count(), 2);
    assert!(implementation.contains("zircon-history-initialize-hdr-pass"));
    assert!(implementation.contains("zircon-history-initialize-ssr-pass"));
    assert!(!implementation.contains("zircon-history-ambient-occlusion"));
    assert!(implementation.contains("let initialization_command_buffer = clear_history_textures("));
    assert!(implementation.contains("initialization_command_buffer,"));
    assert!(implementation.contains("screen_space_reflection.is_none()"));
    assert!(implementation.contains("return None"));
    assert!(implementation.contains("Some(encoder.finish())"));
    assert!(!implementation.contains("submit_graphics_command_buffers("));
    assert!(!implementation.contains("enqueue_graphics_command_buffers("));
    assert!(!implementation.contains("queue.submit("));
}

#[test]
fn every_physical_history_owner_is_guarded_by_compiled_requirements() {
    let source = include_str!("../construct.rs");
    let implementation = source.split("#[cfg(test)]").next().unwrap();
    let constructor = implementation
        .split("pub(crate) fn new_with_requirements_and_initialization(")
        .nth(1)
        .and_then(|tail| {
            tail.split("pub(crate) fn reconcile_with_requirements_and_initialization(")
                .next()
        })
        .expect("initial history constructor");

    for requirement in [
        ".taa_scene_color()",
        ".hybrid_global_illumination()",
        ".screen_space_reflection()",
        ".hzb_furthest()",
        ".exposure()",
        ".volumetric_scattering()",
    ] {
        assert!(implementation.contains(requirement));
    }
    assert_eq!(constructor.matches(".then(||").count(), 5);
    assert!(constructor.contains(".map(|quality| VolumetricHistoryTexture::new("));
}

#[test]
fn reconcile_applies_replacements_only_after_clear_commands_are_encoded() {
    let source = include_str!("../construct.rs");
    let implementation = source.split("#[cfg(test)]").next().unwrap();
    let clear = implementation
        .rfind("let initialization_command_buffer = clear_history_textures(")
        .expect("reconcile must encode clears for newly-created attachments");
    let first_assignment = implementation[clear..]
        .find("self.taa_scene_color = replacement;")
        .map(|offset| clear + offset)
        .expect("reconcile must publish the TAA replacement");
    let commit_requirements = implementation[clear..]
        .find("self.requirements = requirements;")
        .map(|offset| clear + offset)
        .expect("reconcile must publish requirements after clear encoding");

    assert!(clear < first_assignment);
    assert!(first_assignment < commit_requirements);
}
