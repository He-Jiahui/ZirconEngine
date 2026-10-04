use crate::core::framework::render::{
    AntiAliasMode, PostProcessExtract, ProjectionMode, RenderFrameExtract,
    RenderHybridGiCompositePolicy, RenderPostProcessEffectStackSettings,
};
use crate::core::math::{UVec2, Vec3};
use crate::graphics::types::ViewportRenderRegion;

use super::super::super::super::super::post_process_params::PostProcessParams;
use super::super::super::super::super::scene_runtime_feature_flags::SceneRuntimeFeatureFlags;
use super::baked_lighting::baked_lighting;
use super::color_grading::color_grading;

/// 供独立效果节点使用的共享参数打包入口，采用默认 GI 组合策略和同一历史可用性值。
/// 需要区分时间重建/GI 历史及来源策略的最终组合节点应调用完整策略入口。
pub(in crate::graphics::scene::scene_renderer::post_process::resources) fn build_post_process_params(
    viewport_size: UVec2,
    cluster_dimensions: UVec2,
    render_region: ViewportRenderRegion,
    scene_color_origin: [u32; 2],
    extract: &RenderFrameExtract,
    post_process: &PostProcessExtract,
    features: SceneRuntimeFeatureFlags,
    history_available: bool,
    reflection_probe_count: u32,
    hybrid_gi_probe_count: u32,
    scheduled_trace_region_count: u32,
    current_hybrid_gi_lighting_available: bool,
) -> PostProcessParams {
    build_post_process_params_with_hybrid_gi_policy(
        viewport_size,
        cluster_dimensions,
        render_region,
        scene_color_origin,
        extract,
        post_process,
        features,
        history_available,
        history_available,
        reflection_probe_count,
        hybrid_gi_probe_count,
        scheduled_trace_region_count,
        current_hybrid_gi_lighting_available,
        RenderHybridGiCompositePolicy::default(),
    )
}

/// 将视图、编译特性与本帧资源可用性合成共享 CPU/WGSL uniform 契约。
/// viewport 为当前局部阶段尺寸，场景颜色原点独立于物理区域；各有效计数须与实际上传前缀一致。
/// 历史开关仅在编译特性和对应域历史均有效时开启，GI 来源策略防止烘焙与动态照明重复参与。
#[allow(clippy::too_many_arguments)]
pub(in crate::graphics::scene::scene_renderer::post_process::resources) fn build_post_process_params_with_hybrid_gi_policy(
    viewport_size: UVec2,
    cluster_dimensions: UVec2,
    render_region: ViewportRenderRegion,
    scene_color_origin: [u32; 2],
    extract: &RenderFrameExtract,
    post_process: &PostProcessExtract,
    features: SceneRuntimeFeatureFlags,
    temporal_history_available: bool,
    hybrid_gi_history_available: bool,
    reflection_probe_count: u32,
    hybrid_gi_probe_count: u32,
    scheduled_trace_region_count: u32,
    current_hybrid_gi_lighting_available: bool,
    hybrid_gi_composite_policy: RenderHybridGiCompositePolicy,
) -> PostProcessParams {
    let color_grading = color_grading(post_process, features);
    let baked_lighting = baked_lighting(extract, features);
    let effect_stack = post_process.effect_stack;
    let effect_view = effect_view_basis_rows(extract);

    PostProcessParams {
        viewport_and_clusters: [
            viewport_size.x.max(1),
            viewport_size.y.max(1),
            render_region.physical_origin()[0],
            render_region.physical_origin()[1],
        ],
        cluster_dimensions: [
            cluster_dimensions.x.max(1),
            cluster_dimensions.y.max(1),
            scene_color_origin[0],
            scene_color_origin[1],
        ],
        feature_flags: [
            u32::from(features.ssao_enabled),
            u32::from(features.clustered_lighting_enabled),
            u32::from(features.temporal_history_enabled && temporal_history_available),
            reflection_probe_count,
        ],
        lighting_flags: [u32::from(features.contact_shadow_enabled), 0, 0, 0],
        hybrid_gi_counts: [
            hybrid_gi_probe_count,
            scheduled_trace_region_count,
            u32::from(features.hybrid_global_illumination_enabled && hybrid_gi_history_available),
            u32::from(
                features.hybrid_global_illumination_enabled && current_hybrid_gi_lighting_available,
            ),
        ],
        hybrid_gi_source_ledger: [
            hybrid_gi_composite_policy.source_mask(),
            u32::from(hybrid_gi_composite_policy.baked_baseline_weight_q8()),
            u32::from(hybrid_gi_composite_policy.dynamic_weight_q8()),
            u32::from(hybrid_gi_composite_policy.accepts_hybrid_gi_output()),
        ],
        anti_alias: [
            u32::from(
                features.anti_alias_enabled && extract.view.anti_alias.mode == AntiAliasMode::Fxaa,
            ),
            0,
            0,
            0,
        ],
        blends: [
            0.24,
            0.0,
            0.0,
            if features.bloom_enabled {
                post_process.bloom.intensity.max(0.0)
            } else {
                0.0
            },
        ],
        grading: [
            color_grading.exposure.max(0.0),
            color_grading.contrast.max(0.0),
            color_grading.saturation.max(0.0),
            color_grading.gamma.max(0.001),
        ],
        tint_and_probe: [
            color_grading.tint.x.max(0.0),
            color_grading.tint.y.max(0.0),
            color_grading.tint.z.max(0.0),
            if features.reflection_probes_enabled {
                0.35
            } else {
                0.0
            },
        ],
        hybrid_gi_color_and_intensity: [
            0.32,
            0.38,
            0.46,
            if features.hybrid_global_illumination_enabled && hybrid_gi_probe_count > 0 {
                0.4
            } else {
                0.0
            },
        ],
        baked_color_and_intensity: [
            baked_lighting.color.x.max(0.0),
            baked_lighting.color.y.max(0.0),
            baked_lighting.color.z.max(0.0),
            baked_lighting.intensity.max(0.0),
        ],
        effect_flags: effect_flags(effect_stack),
        effect_tonemap_lut: effect_tonemap_lut(effect_stack),
        effect_blur_dof: effect_blur_dof(effect_stack),
        effect_dof_lens: effect_dof_lens(effect_stack),
        effect_vignette_grain: effect_vignette_grain(effect_stack),
        effect_chromatic_fog: effect_chromatic_fog(effect_stack),
        effect_fog_color: effect_fog_color(effect_stack),
        effect_dither_ssr: effect_dither_ssr(effect_stack),
        effect_ssr_limits: effect_ssr_limits(effect_stack),
        effect_depth: effect_depth(extract),
        effect_projection: effect_projection(viewport_size, extract),
        effect_view_x: effect_view[0],
        effect_view_y: effect_view[1],
        effect_view_z: effect_view[2],
        effect_motion_blur: effect_motion_blur(effect_stack),
    }
}

// 效果设置进入 shader 入口选择域；LUT 的最终绑定模式还会由执行层按实际资源重写。
fn effect_flags(settings: RenderPostProcessEffectStackSettings) -> [u32; 4] {
    [
        settings.tonemap.render_operator_id(),
        u32::from(settings.color_lookup.is_enabled()),
        settings.screen_space_reflection.max_steps,
        u32::from(settings.is_enabled()),
    ]
}

// 显示映射参数供 LUT 烘焙和未烘焙的组合路径共用，调用者负责选择匹配的 LUT 模式。
fn effect_tonemap_lut(settings: RenderPostProcessEffectStackSettings) -> [f32; 4] {
    [
        settings.tonemap.render_exposure_bias(),
        settings.tonemap.render_white_point(),
        settings.color_lookup.render_intensity(),
        0.0,
    ]
}

// 共享布局容纳 blur 与 DoF；独立节点和最终组合的 skip 标志决定哪部分由本 pass 消费。
fn effect_blur_dof(settings: RenderPostProcessEffectStackSettings) -> [f32; 4] {
    [
        settings.blur.render_radius(),
        settings.depth_of_field.render_focus_distance(),
        settings.depth_of_field.render_aperture(),
        settings.depth_of_field.render_max_blur_radius(),
    ]
}

// 光学设置供景深预计算/组合契约使用，保持与相机深度和效果侧清洗规则一致。
fn effect_dof_lens(settings: RenderPostProcessEffectStackSettings) -> [f32; 4] {
    [
        settings.depth_of_field.render_focal_length_mm(),
        settings.depth_of_field.render_focus_range(),
        settings.depth_of_field.render_bokeh_blade_count() as f32,
        settings.depth_of_field.bokeh_rotation_radians,
    ]
}

// 运动模糊只在独立节点或最终组合之一生效，执行层可按图拆分结果清零此参数组。
fn effect_motion_blur(settings: RenderPostProcessEffectStackSettings) -> [f32; 4] {
    [
        settings.motion_blur.render_shutter_angle(),
        settings.motion_blur.render_samples() as f32,
        0.0,
        0.0,
    ]
}

// 显示域的暗角和颗粒控制共享一组槽位，字段顺序属于 WGSL 读取契约。
fn effect_vignette_grain(settings: RenderPostProcessEffectStackSettings) -> [f32; 4] {
    [
        settings.vignette.render_intensity(),
        settings.vignette.render_smoothness(),
        settings.vignette.render_roundness(),
        settings.grain.render_intensity(),
    ]
}

// 场景组合与色差共享布局；拆出 scene composite 时执行层仅关闭已独立处理的雾参数。
fn effect_chromatic_fog(settings: RenderPostProcessEffectStackSettings) -> [f32; 4] {
    [
        settings.chromatic_aberration.render_intensity(),
        settings.chromatic_aberration.render_sample_spread(),
        settings.fog.render_density(),
        settings.fog.render_height_falloff(),
    ]
}

// 保持雾颜色和颗粒响应在共享布局的固定槽位，不能按某个单一效果重新解释整个向量。
fn effect_fog_color(settings: RenderPostProcessEffectStackSettings) -> [f32; 4] {
    let fog_color = settings.fog.render_color();

    [
        fog_color.x,
        fog_color.y,
        fog_color.z,
        settings.grain.render_response(),
    ]
}

// 抖动与 SSR 控制共享槽位；独立场景组合已消费 SSR 时，执行层关闭相应分量。
fn effect_dither_ssr(settings: RenderPostProcessEffectStackSettings) -> [f32; 4] {
    [
        settings.dither.render_intensity(),
        settings.dither.render_scale(),
        settings.screen_space_reflection.render_intensity(),
        settings.screen_space_reflection.render_thickness(),
    ]
}

// 为 SSR 搜索和历史解析提供同一组限制，需与独立解析节点及 WGSL 路径保持一致。
fn effect_ssr_limits(settings: RenderPostProcessEffectStackSettings) -> [f32; 4] {
    [
        settings.screen_space_reflection.render_max_ray_distance(),
        settings.screen_space_reflection.max_steps as f32,
        settings
            .screen_space_reflection
            .render_temporal_blend_factor(),
        settings.screen_space_reflection.render_roughness_mip_bias(),
    ]
}

// Shader 据投影类型选择深度线性化，近远范围经过保护后与重建位置辅助参数共同使用。
fn effect_depth(extract: &RenderFrameExtract) -> [f32; 4] {
    let camera = &extract.view.camera;
    let near = camera.z_near.max(0.001);
    let far = camera.z_far.max(near + 0.001);

    [
        near,
        far,
        1.0 / (far - near).max(0.001),
        if matches!(camera.projection_mode, ProjectionMode::Perspective) {
            1.0
        } else {
            0.0
        },
    ]
}

// 为屏幕空间深度/法线重建提供局部视图的投影尺度；显式排除时间抖动。
fn effect_projection(viewport_size: UVec2, extract: &RenderFrameExtract) -> [f32; 4] {
    let camera = &extract.view.camera;
    let aspect = viewport_size.x.max(1) as f32 / viewport_size.y.max(1) as f32;
    let fov_y = camera
        .fov_y_radians
        .clamp(0.001, std::f32::consts::PI - 0.001);
    let focal_y = 1.0 / (fov_y * 0.5).tan().max(0.001);
    let focal_x = focal_y / aspect.max(0.001);
    let half_height = camera.ortho_size.max(0.01);
    let half_width = half_height * aspect.max(0.001);

    [focal_x, focal_y, half_width, half_height]
}

// 将场景相机坐标轴传入屏幕空间法线重建，避免将世界轴误当作视图轴。
fn effect_view_basis_rows(extract: &RenderFrameExtract) -> [[f32; 4]; 3] {
    let transform = extract.view.camera.transform;

    [
        camera_axis_row(transform.right(), Vec3::X),
        camera_axis_row(transform.up(), Vec3::Y),
        camera_axis_row((transform.rotation * Vec3::Z).normalize_or_zero(), Vec3::Z),
    ]
}

// 非有限或退化轴使用规范轴，使后处理的重建基底保持可用；不传播相机时间抖动。
fn camera_axis_row(axis: Vec3, fallback: Vec3) -> [f32; 4] {
    let normalized = if axis.is_finite() && axis.length_squared() > 0.0001 {
        axis.normalize()
    } else {
        fallback
    };

    normalized.extend(0.0).to_array()
}

#[cfg(test)]
#[path = "tests/build.rs"]
mod tests;
