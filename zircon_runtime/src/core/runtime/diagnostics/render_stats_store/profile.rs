use crate::core::framework::render::{
    RenderBudgetKey, RenderPassNativeResourceCreateMetrics, RenderStats,
    RenderSubsystemProfileEntry,
};

use super::{record_bool, record_bytes, record_count, record_microseconds, DiagnosticStore};

pub(super) fn record(store: &mut DiagnosticStore, stats: &RenderStats) {
    let frame_index = stats.submitted_frames;
    let profile = &stats.last_frame_profile;
    record_count(
        store,
        "render.profile.pass_count",
        frame_index,
        profile.passes.len(),
        &["render", "profile"],
    );
    record_microseconds(
        store,
        "render.profile.cpu_submit_time_us",
        frame_index,
        profile.cpu_submit_time_us,
        &["render", "profile", "cpu"],
    );
    record_count(
        store,
        "render.profile.parallel_recording.eligible_stage_count",
        frame_index,
        profile.parallel_recording_eligible_stage_count as usize,
        &["render", "profile", "parallel_recording"],
    );
    record_count(
        store,
        "render.profile.parallel_recording.eligible_bucket_count",
        frame_index,
        profile.parallel_recording_eligible_bucket_count as usize,
        &["render", "profile", "parallel_recording"],
    );
    record_count(
        store,
        "render.profile.parallel_recording.executed_stage_count",
        frame_index,
        profile.parallel_recording_executed_stage_count as usize,
        &["render", "profile", "parallel_recording"],
    );
    record_count(
        store,
        "render.profile.parallel_recording.executed_bucket_count",
        frame_index,
        profile.parallel_recording_executed_bucket_count as usize,
        &["render", "profile", "parallel_recording"],
    );
    // 只有异步 GPU query 已解析并提供 Some 时才记时长；缺样本不等于 0。
    if let Some(gpu_frame_time_us) = profile.gpu_frame_time_us {
        record_microseconds(
            store,
            "render.profile.gpu_frame_time_us",
            frame_index,
            gpu_frame_time_us,
            &["render", "profile", "gpu"],
        );
    }
    record_count(
        store,
        "render.profile.latency_frames",
        frame_index,
        profile.profile_latency_frames as usize,
        &["render", "profile", "gpu"],
    );
    record_bytes(
        store,
        "render.profile.transient_texture_peak_bytes",
        frame_index,
        profile.transient_texture_peak_bytes,
        &["render", "profile", "memory", "texture"],
    );
    record_bytes(
        store,
        "render.profile.transient_buffer_peak_bytes",
        frame_index,
        profile.transient_buffer_peak_bytes,
        &["render", "profile", "memory", "buffer"],
    );
    record_bytes(
        store,
        "render.profile.staging_total_bytes",
        frame_index,
        profile.staging_total_bytes,
        &["render", "profile", "staging"],
    );
    record_bytes(
        store,
        "render.profile.persistent_texture_resident_bytes",
        frame_index,
        profile.persistent_texture_resident_bytes,
        &["render", "profile", "memory", "texture", "persistent"],
    );
    record_bool(
        store,
        "render.profile.compiled_graph_cache_hit",
        frame_index,
        profile.compiled_graph_cache_hit,
        &["render", "profile", "graph", "cache"],
    );
    record_count(
        store,
        "render.profile.variant_miss_count",
        frame_index,
        profile.variant_miss_count as usize,
        &["render", "profile", "shader", "variant"],
    );
    record_count(
        store,
        "render.profile.store_lint_count",
        frame_index,
        profile.store_lint_count as usize,
        &["render", "profile", "store_lint"],
    );
    record_count(
        store,
        "render.profile.budget_warning_count",
        frame_index,
        profile.budget_warning_count as usize,
        &["render", "profile", "budget"],
    );
    record_count(
        store,
        "render.profile.degrade_step_active",
        frame_index,
        profile.degrade_step_active as usize,
        &["render", "profile", "budget", "degrade"],
    );
    record_native_resource_creates(
        store,
        frame_index,
        profile.passes.iter().fold(
            RenderPassNativeResourceCreateMetrics::default(),
            |total, pass| total.saturating_add(pass.native_resource_creates),
        ),
    );

    for subsystem in &profile.subsystems {
        record_subsystem_gpu_timing(store, frame_index, subsystem);
    }
}

fn record_native_resource_creates(
    store: &mut DiagnosticStore,
    frame_index: u64,
    metrics: RenderPassNativeResourceCreateMetrics,
) {
    let paths = [
        (
            "render.profile.native_resource_create.total_count",
            metrics.total_count(),
        ),
        (
            "render.profile.native_resource_create.buffer_count",
            metrics.buffer_count,
        ),
        (
            "render.profile.native_resource_create.bind_group_count",
            metrics.bind_group_count,
        ),
        (
            "render.profile.native_resource_create.bind_group_layout_count",
            metrics.bind_group_layout_count,
        ),
        (
            "render.profile.native_resource_create.shader_module_count",
            metrics.shader_module_count,
        ),
        (
            "render.profile.native_resource_create.pipeline_layout_count",
            metrics.pipeline_layout_count,
        ),
        (
            "render.profile.native_resource_create.compute_pipeline_count",
            metrics.compute_pipeline_count,
        ),
        (
            "render.profile.native_resource_create.render_pipeline_count",
            metrics.render_pipeline_count,
        ),
    ];
    for (path, value) in paths {
        record_count(
            store,
            path,
            frame_index,
            value as usize,
            &["render", "profile", "native_resource_create"],
        );
    }
}

fn record_subsystem_gpu_timing(
    store: &mut DiagnosticStore,
    frame_index: u64,
    subsystem: &RenderSubsystemProfileEntry,
) {
    // 子系统 query 尚未解析时不记录 GPU 时长，也不据此写预算判断。
    let Some(gpu_time_us) = subsystem.gpu_time_us else {
        return;
    };
    let (gpu_path, budget_path, over_budget_path) = subsystem_paths(subsystem.key);
    record_microseconds(
        store,
        gpu_path,
        frame_index,
        gpu_time_us,
        &["render", "profile", "gpu", "subsystem"],
    );
    record_microseconds(
        store,
        budget_path,
        frame_index,
        subsystem.budget_us,
        &["render", "profile", "budget", "subsystem"],
    );
    record_bool(
        store,
        over_budget_path,
        frame_index,
        subsystem.over_budget,
        &["render", "profile", "budget", "subsystem"],
    );
}

macro_rules! subsystem_path_set {
    ($name:ident) => {
        (
            concat!(
                "render.profile.subsystem.",
                stringify!($name),
                ".gpu_time_us"
            ),
            concat!("render.profile.subsystem.", stringify!($name), ".budget_us"),
            concat!(
                "render.profile.subsystem.",
                stringify!($name),
                ".over_budget"
            ),
        )
    };
}

fn subsystem_paths(key: RenderBudgetKey) -> (&'static str, &'static str, &'static str) {
    match key {
        RenderBudgetKey::Shadow => subsystem_path_set!(shadow),
        RenderBudgetKey::DepthPrepass => subsystem_path_set!(depth_prepass),
        RenderBudgetKey::Hzb => subsystem_path_set!(hzb),
        RenderBudgetKey::GpuSceneUpdate => subsystem_path_set!(gpu_scene_update),
        RenderBudgetKey::BasePass => subsystem_path_set!(base_pass),
        RenderBudgetKey::LightGrid => subsystem_path_set!(light_grid),
        RenderBudgetKey::DeferredLighting => subsystem_path_set!(deferred_lighting),
        RenderBudgetKey::Ssao => subsystem_path_set!(ssao),
        RenderBudgetKey::Transparent => subsystem_path_set!(transparent),
        RenderBudgetKey::PostProcess => subsystem_path_set!(post_process),
        RenderBudgetKey::TemporalAa => subsystem_path_set!(temporal_aa),
        RenderBudgetKey::Ui => subsystem_path_set!(ui),
        RenderBudgetKey::Other => subsystem_path_set!(other),
    }
}

#[cfg(test)]
#[path = "tests/profile.rs"]
mod tests;
