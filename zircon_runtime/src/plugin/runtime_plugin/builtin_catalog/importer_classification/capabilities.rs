const IMPORTER_CAPABILITY_PREFIX: &str = "runtime.asset.importer.";

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
mod tests {
    use super::primary_importer_capability;

    #[test]
    fn fallback_importer_capability_writes_slug_into_single_output() {
        assert_eq!(
            primary_importer_capability("custom_mesh_importer"),
            "runtime.asset.importer.custom.mesh"
        );
        assert_eq!(
            primary_importer_capability("procedural_cache"),
            "runtime.asset.importer.procedural.cache"
        );
        assert_eq!(
            primary_importer_capability("gltf_importer"),
            "runtime.asset.importer.model.gltf"
        );
    }
}
