use super::super::super::RuntimePluginDescriptor;

pub(super) fn assign_category(descriptor: RuntimePluginDescriptor) -> RuntimePluginDescriptor {
    match descriptor.package_id.as_str() {
        "texture" => descriptor.with_category("runtime"),
        "terrain" | "tilemap_2d" | "prefab_tools" => descriptor.with_category("authoring"),
        "virtual_geometry" | "hybrid_gi" | "solari" => descriptor.with_category("rendering"),
        "gltf_importer"
        | "obj_importer"
        | "texture_importer"
        | "audio_importer"
        | "shader_wgsl_importer"
        | "ui_document_importer" => descriptor.with_category("asset_importer"),
        "rendering" => descriptor.with_category("rendering"),
        _ => descriptor,
    }
}
