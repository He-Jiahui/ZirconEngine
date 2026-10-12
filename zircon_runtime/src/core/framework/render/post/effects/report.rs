use super::{
    resource_status::RenderPostProcessEffectStackResourceStatus,
    RenderPostProcessEffectStackSettings,
};
use crate::core::framework::render::MotionVectorCameraStatus;

/// 面向统计与调试界面的效果族诊断：区分已请求、近似实现及运行时资源缺口。
/// 调用方应提供当前帧的资源状态；默认资源状态只适合不掌握 GPU 上下文的静态预览。
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct RenderPostProcessEffectStackReport {
    pub enabled: bool,
    pub active_family_count: usize,
    pub active_families: Vec<String>,
    pub approximated_family_count: usize,
    pub approximated_families: Vec<String>,
    pub missing_resource_count: usize,
    pub missing_resources: Vec<String>,
}

impl RenderPostProcessEffectStackReport {
    pub fn from_settings(settings: RenderPostProcessEffectStackSettings) -> Self {
        Self::from_settings_with_resources(
            settings,
            RenderPostProcessEffectStackResourceStatus::default(),
        )
    }

    pub fn from_settings_with_resources(
        settings: RenderPostProcessEffectStackSettings,
        resources: RenderPostProcessEffectStackResourceStatus,
    ) -> Self {
        let mut report = Self::default();
        let depth_of_field_enabled = settings.depth_of_field.is_enabled();
        let motion_blur_enabled = settings.motion_blur.is_enabled();
        let screen_space_reflection_enabled = settings.screen_space_reflection.is_enabled();

        push_label(
            &mut report.active_families,
            settings.tonemap.is_enabled(),
            "tonemap",
        );
        push_label(
            &mut report.active_families,
            settings.color_lookup.is_enabled(),
            "lut",
        );
        push_label(
            &mut report.active_families,
            settings.blur.is_enabled(),
            "blur",
        );
        push_label(
            &mut report.active_families,
            depth_of_field_enabled,
            "depth-of-field",
        );
        push_label(
            &mut report.active_families,
            motion_blur_enabled,
            "motion-blur",
        );
        push_label(
            &mut report.active_families,
            screen_space_reflection_enabled,
            "screen-space-reflection",
        );
        push_label(
            &mut report.active_families,
            settings.vignette.is_enabled(),
            "vignette",
        );
        push_label(
            &mut report.active_families,
            settings.grain.is_enabled(),
            "film-grain",
        );
        push_label(
            &mut report.active_families,
            settings.dither.is_enabled(),
            "dither",
        );
        push_label(
            &mut report.active_families,
            settings.chromatic_aberration.is_enabled(),
            "chromatic-aberration",
        );
        push_label(
            &mut report.active_families,
            settings.fog.is_enabled(),
            "fog",
        );

        push_label(
            &mut report.approximated_families,
            depth_of_field_enabled,
            "depth-of-field",
        );
        push_label(
            &mut report.approximated_families,
            motion_blur_enabled,
            "motion-blur",
        );
        push_label(
            &mut report.approximated_families,
            screen_space_reflection_enabled,
            "screen-space-reflection",
        );

        if settings.color_lookup.intensity > 0.0 && settings.color_lookup.texture.is_none() {
            report
                .missing_resources
                .push("effect-stack.lut.texture".to_string());
        }
        if settings.color_lookup.intensity > 0.0
            && !settings
                .color_lookup
                .texture_layout
                .has_valid_requested_size()
        {
            report
                .missing_resources
                .push("effect-stack.lut.texture-layout".to_string());
        }
        if screen_space_reflection_enabled && !resources.ssr_normal_available {
            report
                .missing_resources
                .push("effect-stack.ssr.normal".to_string());
        }
        let ssr_temporal_history_missing =
            screen_space_reflection_enabled && !resources.ssr_temporal_history_available;
        if ssr_temporal_history_missing {
            report
                .missing_resources
                .push("effect-stack.ssr.temporal-history".to_string());
        }
        let ssr_temporal_vector_missing = screen_space_reflection_enabled
            && resources.ssr_temporal_history_available
            && !resources.motion_vector_available;
        if ssr_temporal_vector_missing {
            report
                .missing_resources
                .push("effect-stack.ssr.temporal-motion-vector".to_string());
        }
        let ssr_temporal_prepass_missing = screen_space_reflection_enabled
            && resources.ssr_temporal_history_available
            && !resources.motion_vector_prepass_available;
        if ssr_temporal_prepass_missing {
            push_velocity_prepass_missing_resources(
                &mut report,
                resources,
                "effect-stack.ssr.temporal-velocity-prepass",
            );
        }
        if motion_blur_enabled && !resources.motion_vector_available {
            report
                .missing_resources
                .push("effect-stack.motion-blur.motion-vector".to_string());
        }
        let motion_vector_prepass_missing =
            motion_blur_enabled && !resources.motion_vector_prepass_available;
        if motion_vector_prepass_missing {
            push_velocity_prepass_missing_resources(
                &mut report,
                resources,
                "effect-stack.motion-blur.velocity-prepass",
            );
        }

        report.enabled = !report.active_families.is_empty();
        report.active_family_count = report.active_families.len();
        report.approximated_family_count = report.approximated_families.len();
        report.missing_resource_count = report.missing_resources.len();
        report
    }
}

fn push_label(labels: &mut Vec<String>, enabled: bool, label: &str) {
    if enabled {
        labels.push(label.to_string());
    }
}

fn push_velocity_prepass_missing_resources(
    report: &mut RenderPostProcessEffectStackReport,
    resources: RenderPostProcessEffectStackResourceStatus,
    prefix: &str,
) {
    report.missing_resources.push(prefix.to_string());
    if !resources.motion_vector_camera_available {
        report.missing_resources.push(format!("{prefix}.camera"));
    }
    if !resources.motion_vector_object_available {
        report.missing_resources.push(format!("{prefix}.object"));
    }
    if resources.motion_vector_camera_status == MotionVectorCameraStatus::MissingPreviousCamera {
        report
            .missing_resources
            .push(format!("{prefix}.camera-history"));
    }
    if resources.motion_vector_camera_status == MotionVectorCameraStatus::CameraCutOrInvalid {
        report
            .missing_resources
            .push(format!("{prefix}.camera-cut-or-invalid"));
    }
    if !resources.motion_vector_tile_max_available {
        report.missing_resources.push(format!("{prefix}.tile-max"));
    }
    if !resources.motion_vector_tile_max_coarse_available {
        report
            .missing_resources
            .push(format!("{prefix}.tile-max-coarse"));
    }
    if !resources.motion_vector_neighbor_max_available {
        report
            .missing_resources
            .push(format!("{prefix}.neighbor-max"));
    }
}

#[cfg(test)]
#[path = "tests/report.rs"]
mod tests;
