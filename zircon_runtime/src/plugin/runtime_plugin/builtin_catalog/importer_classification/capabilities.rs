pub(super) fn primary_importer_capability(package_id: &str) -> String {
    match package_id {
        "gltf_importer" => return "runtime.asset.importer.model.gltf".to_string(),
        "obj_importer" => return "runtime.asset.importer.model.obj".to_string(),
        "audio_importer" => return "runtime.asset.importer.audio.wav".to_string(),
        "shader_wgsl_importer" => return "runtime.asset.importer.shader.wgsl".to_string(),
        "ui_document_importer" => return "runtime.asset.importer.ui_document".to_string(),
        _ => {}
    }
    let slug = package_id
        .strip_suffix("_importer")
        .unwrap_or(package_id)
        .replace('_', ".");
    format!("runtime.asset.importer.{slug}")
}
