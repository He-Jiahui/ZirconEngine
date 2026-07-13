use crate::core::framework::render::{
    FroxelGridQuality, RenderFrameExtract, ShaderQualityTier, VolumetricFogSettings,
};

pub(crate) fn resolved_volumetric_fog_settings(
    extract: &RenderFrameExtract,
) -> Result<VolumetricFogSettings, String> {
    if let Some(settings) = extract.lighting.advanced_lighting.volumetric {
        return Ok(settings);
    }

    let camera = extract.view.selected_effective_camera();
    extract
        .post_process
        .resolved_settings_for_camera(
            camera.transform.translation,
            extract.view.selected_camera_volume_layers(),
        )
        .map(|settings| settings.volumetric_fog)
        .map_err(|error| format!("volumetric fog volume evaluation failed: {error:?}"))
}

pub(crate) fn volumetric_history_quality(
    extract: &RenderFrameExtract,
    shader_quality: ShaderQualityTier,
) -> Result<Option<FroxelGridQuality>, String> {
    let quality = FroxelGridQuality::from_shader_quality(shader_quality);
    if !quality.supports_temporal() {
        return Ok(None);
    }
    Ok(resolved_volumetric_fog_settings(extract)?
        .temporal
        .then_some(quality))
}
