use crate::core::framework::render::{RenderBudgetKey, RenderStats, RenderSubsystemProfileEntry};

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

    for subsystem in &profile.subsystems {
        record_subsystem_gpu_timing(store, frame_index, subsystem);
    }
}

fn record_subsystem_gpu_timing(
    store: &mut DiagnosticStore,
    frame_index: u64,
    subsystem: &RenderSubsystemProfileEntry,
) {
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
mod tests {
    use std::sync::Arc;

    use crate::core::framework::render::{
        RenderBudgetKey, RenderFrameProfile, RenderStats, RenderSubsystemProfileEntry,
    };
    use crate::core::runtime::diagnostics::DiagnosticStore;

    use super::record;

    #[test]
    fn profile_diagnostics_mirror_resolved_gpu_budget_data_without_dynamic_pass_paths() {
        let mut store = DiagnosticStore::default();
        let stats = RenderStats {
            submitted_frames: 12,
            last_frame_profile: Arc::new(RenderFrameProfile {
                cpu_submit_time_us: 1_500,
                gpu_frame_time_us: Some(4_000),
                profile_latency_frames: 2,
                budget_warning_count: 1,
                subsystems: vec![RenderSubsystemProfileEntry {
                    key: RenderBudgetKey::BasePass,
                    gpu_time_us: Some(4_000),
                    budget_us: 3_200,
                    over_budget: true,
                }],
                ..RenderFrameProfile::default()
            }),
            ..RenderStats::default()
        };

        record(&mut store, &stats);

        assert_series(
            &store,
            "render.profile.cpu_submit_time_us",
            1_500.0,
            "microseconds",
        );
        assert_series(
            &store,
            "render.profile.gpu_frame_time_us",
            4_000.0,
            "microseconds",
        );
        assert_series(
            &store,
            "render.profile.subsystem.base_pass.gpu_time_us",
            4_000.0,
            "microseconds",
        );
        assert_series(
            &store,
            "render.profile.subsystem.base_pass.over_budget",
            1.0,
            "bool",
        );
        assert!(store
            .snapshot()
            .series
            .iter()
            .all(|series| !series.path.as_str().contains("opaque")));
    }

    fn assert_series(store: &DiagnosticStore, path: &str, value: f64, unit: &str) {
        let series = store
            .snapshot()
            .series
            .into_iter()
            .find(|series| series.path.as_str() == path)
            .unwrap_or_else(|| panic!("missing diagnostic series `{path}`"));
        assert_eq!(series.current, Some(value));
        assert_eq!(series.unit.as_deref(), Some(unit));
    }
}
