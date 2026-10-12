use crate::core::framework::render::{
    FroxelGridQuality, RenderFrameExtract, ShaderQualityTier, VolumetricFogSettings,
};

/// 优先采用帧提取时确定的高级光照设置；仅缺失时按选中相机位置和体积层重新求值，
/// 供帧历史准备阶段确定体积雾是否需要历史资源。
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

/// 只有画质支持历史且本帧体积雾允许时间累积时才请求历史纹理。
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
