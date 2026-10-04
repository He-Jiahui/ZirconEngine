const IMPORTER_CAPABILITY_PREFIX: &str = "runtime.asset.importer.";

// 已发布的导入器能力使用固定标识；其余包名才走后缀剥离与下划线转换。
pub(super) fn primary_importer_capability(package_id: &str) -> String {
    match package_id {
        "gltf_importer" => return "runtime.asset.importer.model.gltf".to_string(),
        "obj_importer" => return "runtime.asset.importer.model.obj".to_string(),
        "audio_importer" => return "runtime.asset.importer.audio.wav".to_string(),
        "shader_wgsl_importer" => return "runtime.asset.importer.shader.wgsl".to_string(),
        "ui_document_importer" => return "runtime.asset.importer.ui_document".to_string(),
        _ => {}
    }
    let slug = package_id.strip_suffix("_importer").unwrap_or(package_id);
    let capacity = IMPORTER_CAPABILITY_PREFIX.len().saturating_add(slug.len());
    let mut capability = String::with_capacity(capacity);
    capability.push_str(IMPORTER_CAPABILITY_PREFIX);
    for character in slug.chars() {
        match character {
            '_' => capability.push('.'),
            character => capability.push(character),
        }
    }
    capability
}

#[cfg(test)]
#[path = "tests/capabilities.rs"]
mod tests;
