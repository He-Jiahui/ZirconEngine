use super::super::*;
use crate::ui::retained_host::host_contract::globals::UiHostContext;
use crate::ui::retained_host::ui_perf::{
    enter_ui_perf_scenario, time_ui_perf_scenario, UiPerfScenario,
};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::OnceLock;
use std::time::{Duration, Instant};

const SLOW_TICK_TRACE_THRESHOLD: Duration = Duration::from_millis(100);
const SLOW_TICK_TRACE_LOG_LIMIT: usize = 64;
static SLOW_TICK_TRACE_ENABLED: OnceLock<bool> = OnceLock::new();
static SLOW_TICK_TRACE_LOG_COUNT: AtomicUsize = AtomicUsize::new(0);
static SLOW_COMMIT_TRACE_LOG_COUNT: AtomicUsize = AtomicUsize::new(0);

fn slow_tick_trace_enabled() -> bool {
    *SLOW_TICK_TRACE_ENABLED.get_or_init(|| {
        matches!(
            std::env::var("ZIRCON_EDITOR_TRACE_SLOW_TICK").as_deref(),
            Ok("1")
        )
    })
}

fn trace_elapsed_ms(start: Instant, end: Instant) -> f64 {
    end.duration_since(start).as_secs_f64() * 1_000.0
}

fn trace_slow_tick(
    started: Option<Instant>,
    refresh_started: Option<Instant>,
    refresh_ended: Option<Instant>,
    commit_ended: Option<Instant>,
) {
    let (Some(started), Some(refresh_started), Some(refresh_ended), Some(commit_ended)) =
        (started, refresh_started, refresh_ended, commit_ended)
    else {
        return;
    };
    let ended = Instant::now();
    if ended.duration_since(started) <= SLOW_TICK_TRACE_THRESHOLD
        || SLOW_TICK_TRACE_LOG_COUNT
            .fetch_update(Ordering::Relaxed, Ordering::Relaxed, |count| {
                (count < SLOW_TICK_TRACE_LOG_LIMIT).then(|| count + 1)
            })
            .is_err()
    {
        return;
    }
    eprintln!(
        "[zircon_editor] slow_tick elapsed_ms={:.2} before_refresh_ms={:.2} refresh_ms={:.2} commit_ms={:.2} after_commit_ms={:.2}",
        trace_elapsed_ms(started, ended),
        trace_elapsed_ms(started, refresh_started),
        trace_elapsed_ms(refresh_started, refresh_ended),
        trace_elapsed_ms(refresh_ended, commit_ended),
        trace_elapsed_ms(commit_ended, ended),
    );
}

impl RetainedEditorHost {
    fn runtime_frame_owner_key(&self) -> Option<(crate::core::play::PlayInstanceId, u64)> {
        if !self.runtime.runtime_event_consumer_session_active() {
            return None;
        }
        let Some(crate::core::play::WorldDomain::Play(instance)) =
            self.runtime.play_sessions().attached_world_domain()
        else {
            return None;
        };
        self.runtime
            .play_sessions()
            .play_gateway(instance)
            .map(|gateway| (instance, gateway.generation()))
    }

    pub(in crate::ui::retained_host::app) fn tick(&mut self) {
        let slow_tick_started = slow_tick_trace_enabled().then(Instant::now);
        zircon_runtime::profile_frame!("editor", "retained_host_tick");
        zircon_runtime::profile_scope!("editor", "retained_host", "tick");
        self.pump_editor_job_events();
        let plugin_watch_deadline = self.poll_module_plugin_development_watches();
        if let Err(error) = self.editor_manager.pump_runtime_task_diagnostics(0) {
            self.set_status_line(error.to_string());
        }
        if let Err(error) = self.editor_manager.pump_project_recovery_decisions() {
            self.set_status_line(error.to_string());
        }
        if let Err(error) = self
            .editor_manager
            .refresh_project_session_heartbeat_if_due(Instant::now())
        {
            let message = error.to_string();
            let entry = LogEntry::new(
                LogSource::editor(),
                LogSeverity::Error,
                message.clone(),
                0,
                None,
            );
            if let Ok(entry) = entry {
                let _ = self.runtime.context().logs().emit(entry);
            }
            self.set_status_line(message);
        }
        let lifecycle_deadline = [
            plugin_watch_deadline,
            self.editor_manager.project_session_heartbeat_deadline(),
        ]
        .into_iter()
        .flatten()
        .min();
        self.ui.set_lifecycle_frame_update(lifecycle_deadline);
        self.poll_editor_autosave();
        self.poll_model_import();
        self.poll_asset_deletion();
        self.poll_asset_relocation();
        self.poll_active_scene_reload();
        self.poll_prompted_close_save();
        self.poll_document_save_all();
        self.poll_welcome_project_probe();
        self.poll_desktop_export_jobs();
        self.poll_desktop_export_wizard_sessions();
        self.sync_editor_job_progress();
        if let Err(error) = self.runtime.pump_plugin_lifecycle_messages() {
            self.set_status_line(error);
        }
        self.runtime.update_scene_modes();
        self.sync_play_preview_input_focus();
        self.sync_simulate_preview_camera();
        let frame_owner = self.runtime_frame_owner_key();
        self.ui.set_runtime_frame_owner(frame_owner);
        let frame_result = self.runtime.pump_runtime_event_consumers();
        if let Err(error) = self.ui.complete_runtime_frame_tick(
            frame_result,
            frame_owner,
            self.runtime_frame_owner_key(),
            Instant::now(),
        ) {
            self.set_status_line(error.to_string());
        }
        self.runtime.sync_active_selection_world_domain();
        self.sync_active_hierarchy_world();
        self.poll_play_viewport_pick_for_native_host();
        self.sync_active_play_inspector();
        self.poll_play_preview_frame_for_native_host();
        if let Err(error) = self.sync_plugin_template_documents_if_changed() {
            self.set_status_line(error.to_string());
        }
        self.sync_activity_notifications();
        self.sync_settings_projections();
        self.tick_workbench_tooltip();

        let refresh_started = slow_tick_started.map(|_| Instant::now());
        {
            let _ui_perf_scenario = enter_ui_perf_scenario(UiPerfScenario::AssetRefresh);
            let _ui_perf_timer = time_ui_perf_scenario(UiPerfScenario::AssetRefresh);
            if let Err(error) = self.refresh_project_assets() {
                self.set_status_line(error);
            }
        }

        let refresh_ended = slow_tick_started.map(|_| Instant::now());
        self.commit_pending_frame_update();
        let commit_ended = slow_tick_started.map(|_| Instant::now());

        {
            let _ui_perf_scenario = enter_ui_perf_scenario(UiPerfScenario::ViewportImage);
            let _ui_perf_timer = time_ui_perf_scenario(UiPerfScenario::ViewportImage);
            self.poll_viewport_image_for_native_host();
        }
        if let Some(error) = self.viewport.take_error() {
            self.set_status_line(error);
            self.recompute_if_dirty();
        }
        trace_slow_tick(
            slow_tick_started,
            refresh_started,
            refresh_ended,
            commit_ended,
        );
    }

    pub(in crate::ui::retained_host::app) fn commit_interactive_frame_update(&mut self) {
        zircon_runtime::profile_scope!(
            "editor",
            "retained_host",
            "commit_interactive_frame_update"
        );
        self.commit_pending_frame_update();
        zircon_runtime::profile_counter!(
            "editor",
            "ui.interactive_frame.maintenance_deferred_count",
            1
        );
        self.ui.set_lifecycle_frame_update(Some(Instant::now()));
    }

    fn commit_pending_frame_update(&mut self) {
        let trace_started = slow_tick_trace_enabled().then(Instant::now);
        let frame_scenario = self.pending_ui_perf_scenario.take();
        let _frame_scenario_guard = frame_scenario.map(enter_ui_perf_scenario);
        if let Some(scenario) = frame_scenario {
            self.ui.mark_completed_frame_update_scenario(scenario);
        }

        self.sync_shell_size();
        let sync_ended = trace_started.map(|_| Instant::now());
        self.recompute_if_dirty();
        let recompute_ended = trace_started.map(|_| Instant::now());
        self.submit_render_frame_if_dirty();
        if let (Some(started), Some(sync_ended), Some(recompute_ended)) =
            (trace_started, sync_ended, recompute_ended)
        {
            let ended = Instant::now();
            if ended.duration_since(started) > SLOW_TICK_TRACE_THRESHOLD
                && SLOW_COMMIT_TRACE_LOG_COUNT
                    .fetch_update(Ordering::Relaxed, Ordering::Relaxed, |count| {
                        (count < SLOW_TICK_TRACE_LOG_LIMIT).then(|| count + 1)
                    })
                    .is_ok()
            {
                eprintln!(
                    "[zircon_editor] slow_commit_frame elapsed_ms={:.2} sync_shell_ms={:.2} recompute_ms={:.2} render_submit_ms={:.2}",
                    trace_elapsed_ms(started, ended),
                    trace_elapsed_ms(started, sync_ended),
                    trace_elapsed_ms(sync_ended, recompute_ended),
                    trace_elapsed_ms(recompute_ended, ended),
                );
            }
        }
    }

    fn sync_play_preview_input_focus(&mut self) {
        let active = self.runtime.play_preview_input_active();
        if active && !self.play_preview_input_focus_active {
            self.ui.global::<UiHostContext>().clear_text_input_focus();
        }
        let view_focused = active && self.runtime.play_preview_view_focused();
        if active && self.play_preview_view_focus_active && !view_focused {
            self.route_play_preview_focus_lost();
        }
        self.play_preview_input_focus_active = active;
        self.play_preview_view_focus_active = view_focused;
    }

    pub(in crate::ui::retained_host::app) fn refresh_ui(&mut self) {
        self.recompute_if_dirty();
    }

    pub(in crate::ui::retained_host::app) fn use_committed_pointer_layout(&self) {
        // Pointer routing must stay on the last committed bridge frames. Dirty
        // presentation/layout state is consumed by tick/refresh instead of
        // rebuilding the whole editor tree inside native pointer callbacks.
        self.publish_refresh_invalidation_diagnostics();
    }

    // Retained host tick owns publishing worker job events into the editor bus.
    fn pump_editor_job_events(&self) {
        self.editor_manager.context().jobs().pump_events();
    }
}

#[cfg(test)]
#[path = "tests/tick.rs"]
mod tests;
