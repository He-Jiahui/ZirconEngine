use super::super::BuiltinCatalogDescriptorBuilder;

pub(super) fn attach_extra_capabilities(
    descriptor: BuiltinCatalogDescriptorBuilder,
) -> BuiltinCatalogDescriptorBuilder {
    let package_id = descriptor.package_id().to_string();
    match package_id.as_str() {
        "ai" => descriptor
            .with_capability("runtime.feature.ai.behavior_tree")
            .with_capability("runtime.feature.ai.blackboard")
            .with_capability("runtime.feature.ai.perception"),
        "animation" => descriptor.with_capability("runtime.feature.animation.timeline_event_track"),
        "zr_vm_language" => descriptor.with_capability("runtime.script.backend.zr_vm_project"),
        "physics" => descriptor
            .with_capability("runtime.capability.physics.raycast")
            .with_capability("runtime.capability.physics.overlap")
            .with_capability("runtime.capability.physics.shape_cast")
            .with_capability("runtime.capability.physics.trigger_events")
            .with_capability("runtime.capability.physics.constraints")
            .with_capability("runtime.capability.physics.skeletal_joints"),
        "gltf_importer" => descriptor.with_capability("runtime.asset.importer.model.gltf"),
        "obj_importer" => descriptor.with_capability("runtime.asset.importer.model.obj"),
        "texture_importer" => descriptor.with_capability("runtime.asset.importer.texture.image"),
        "audio_importer" => descriptor.with_capability("runtime.asset.importer.audio.wav"),
        "shader_wgsl_importer" => descriptor.with_capability("runtime.asset.importer.shader.wgsl"),
        "ui_document_importer" => descriptor.with_capability("runtime.asset.importer.ui_document"),
        _ => descriptor,
    }
}
