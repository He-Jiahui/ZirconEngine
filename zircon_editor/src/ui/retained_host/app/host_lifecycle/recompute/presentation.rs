use std::collections::BTreeSet;

use super::super::super::RetainedEditorHost;
use crate::ui::retained_host::app::committed_shell_state::HostLifecyclePanePayloads;
use crate::ui::retained_host::callback_dispatch;
use crate::ui::retained_host::floating_window_projection::FloatingWindowProjectionBundle;
use crate::ui::retained_host::host_contract::{FrameRect, TemplatePaneNodeData};
use crate::ui::retained_host::ui::{
    apply_presentation_with_template_v2_data,
    build_host_contract_workbench_window_node_patch_at_mount_and_scale,
    build_ui_asset_presentation_patch, patch_shell_content_presentation_from_state,
    patch_ui_asset_presentation, shell_content_target,
};
use crate::ui::retained_host::ui_perf::{
    enter_ui_perf_scenario, record_current_ui_perf_counter, time_ui_perf_scenario, UiPerfCounter,
    UiPerfScenario,
};
use crate::ui::retained_host::HostShellContentScope;
use crate::ui::workbench::autolayout::WorkbenchShellGeometry;
use crate::ui::workbench::model::WorkbenchViewModel;
use crate::ui::workbench::snapshot::EditorChromeSnapshot;
use crate::ui::workbench::view::ViewInstanceId;

impl RetainedEditorHost {
    pub(super) fn apply_committed_shell_content_presentation(
        &mut self,
        scope: &HostShellContentScope,
    ) -> bool {
        zircon_runtime::profile_scope!(
            "editor",
            "retained_host",
            "apply_committed_shell_content_presentation"
        );
        let _ui_perf_scenario = enter_ui_perf_scenario(UiPerfScenario::ShellContent);
        let _ui_perf_timer = time_ui_perf_scenario(UiPerfScenario::ShellContent);
        let Some(mut committed) = self.committed_shell_state.take() else {
            zircon_runtime::profile_counter!(
                "editor",
                "ui.shell_content.committed_state_miss_count",
                1
            );
            return false;
        };
        let next_layout = self.runtime.current_layout();
        if !committed.patch_shell_content(scope, next_layout) {
            zircon_runtime::profile_counter!(
                "editor",
                "ui.shell_content.committed_state_validation_fallback_count",
                1
            );
            return false;
        }
        let applied = self.apply_shell_content_presentation(
            scope,
            &committed.model,
            &committed.chrome,
            &committed.geometry,
            committed.layout_frames,
        );
        if !applied {
            return false;
        }
        self.runtime_diagnostics_refresh_target = crate::ui::retained_host::app::runtime_diagnostics_visibility::runtime_diagnostics_refresh_target(
            &committed.model,
        );
        committed.pane_payloads = None;
        committed.retained_shell_presentation = None;
        self.shell_geometry = Some(committed.geometry.clone());
        self.committed_shell_state = Some(committed);
        zircon_runtime::profile_counter!("editor", "ui.shell_content.committed_state_hit_count", 1);
        true
    }

    pub(super) fn apply_shell_content_presentation(
        &mut self,
        scope: &HostShellContentScope,
        model: &WorkbenchViewModel,
        chrome: &EditorChromeSnapshot,
        geometry: &WorkbenchShellGeometry,
        componentized_workbench_layout_frames: callback_dispatch::BuiltinWorkbenchWindowLayoutFrames,
    ) -> bool {
        zircon_runtime::profile_scope!(
            "editor",
            "retained_host",
            "apply_early_shell_content_presentation"
        );
        let Some(target) = shell_content_target(scope, model) else {
            return false;
        };
        let pane_payloads = self.collect_shell_content_pane_payloads(
            chrome,
            target.content_kind,
            target.instance_id.as_deref(),
        );
        let editor_payload_complete = match target.content_kind {
            crate::ui::workbench::snapshot::ViewContentKind::UiAssetEditor => target
                .instance_id
                .as_ref()
                .is_some_and(|instance_id| pane_payloads.ui_asset_panes.contains_key(instance_id)),
            crate::ui::workbench::snapshot::ViewContentKind::AnimationSequenceEditor
            | crate::ui::workbench::snapshot::ViewContentKind::AnimationGraphEditor => target
                .instance_id
                .as_ref()
                .is_some_and(|instance_id| pane_payloads.animation_panes.contains_key(instance_id)),
            _ => true,
        };
        if !editor_payload_complete {
            zircon_runtime::profile_counter!(
                "editor",
                "ui.shell_content.target_payload_missing_count",
                1
            );
            return false;
        }
        let filtered_hierarchy_entries =
            shell_content_requires_hierarchy_filter(target.content_kind)
                .then(|| self.filtered_hierarchy_entries(&chrome.scene_entries))
                .flatten();
        let filtered_chrome = filtered_hierarchy_entries.map(|scene_entries| {
            let mut chrome = chrome.clone();
            chrome.scene_entries = scene_entries;
            chrome
        });
        let chrome = filtered_chrome.as_ref().unwrap_or(chrome);
        let has_component_showcase_runtime =
            shell_content_requires_component_runtime(target.content_kind)
                && self.prepare_component_showcase_runtime_for_presentation(model);
        let pane_template_runtime = if has_component_showcase_runtime {
            &self.component_showcase_runtime
        } else {
            self.builtin_template_runtime.as_ref()
        };
        let hierarchy_filter_query = self.hierarchy_filter_query.clone();
        patch_shell_content_presentation_from_state(
            &self.ui,
            target,
            model,
            chrome,
            geometry,
            &pane_payloads.preset_names,
            self.active_layout_preset.as_deref(),
            &pane_payloads.ui_asset_panes,
            &pane_payloads.animation_panes,
            &pane_payloads.runtime_diagnostics,
            &pane_payloads.module_plugins,
            &pane_payloads.build_export,
            &pane_payloads.template_v2_data,
            componentized_workbench_layout_frames,
            Some(pane_template_runtime),
            &hierarchy_filter_query,
            &mut self.host_chrome_projection_cache,
            &mut self.console_pane_projection_cache,
            &mut self.module_plugins_pane_projection_cache,
        )
    }

    pub(super) fn apply_workbench_projection_presentation(&mut self) -> bool {
        zircon_runtime::profile_scope!(
            "editor",
            "retained_host",
            "apply_workbench_projection_presentation_patch"
        );
        let Some(projection_nodes) = self
            .workbench_window_bridge
            .pending_host_projection_patch_nodes()
        else {
            record_workbench_projection_fallback(WorkbenchProjectionFallback::FullProjection);
            return false;
        };
        if projection_nodes.is_empty() {
            self.workbench_window_bridge
                .mark_host_projection_committed();
            zircon_runtime::profile_counter!(
                "editor",
                "ui.workbench_projection.presentation_patch_noop_count",
                1
            );
            self.publish_refresh_invalidation_diagnostics();
            return true;
        }

        let current_generation = self.ui.get_host_presentation_generation();
        let current_nodes = &current_generation.structure().workbench_window_nodes;
        let Some(patch) = build_host_contract_workbench_window_node_patch_at_mount_and_scale(
            &self.workbench_window_bridge.host_projection().document_id,
            &projection_nodes,
            current_nodes,
            self.workbench_window_bridge.layout_frames().mount_frame,
            self.workbench_window_bridge.presentation_scale_factor(),
        ) else {
            record_workbench_projection_fallback(WorkbenchProjectionFallback::Projection);
            return false;
        };
        let damage = workbench_projection_damage(current_nodes, &patch.nodes, &patch.changed_rows);
        drop(current_generation);
        let patched_node_count = patch.changed_rows.len();
        if !self
            .ui
            .patch_workbench_window_nodes(patch.nodes, &patch.changed_rows)
        {
            record_workbench_projection_fallback(WorkbenchProjectionFallback::HitIndex);
            return false;
        }
        self.invalidate_committed_shell_presentation();
        self.workbench_window_bridge
            .mark_host_projection_committed();
        for frame in &damage {
            self.ui.request_frame_update_region(frame.clone());
        }
        zircon_runtime::profile_counter!(
            "editor",
            "ui.workbench_projection.presentation_patch_count",
            1
        );
        zircon_runtime::profile_counter!(
            "editor",
            "ui.workbench_projection.presentation_patch_node_count",
            patched_node_count
        );
        zircon_runtime::profile_counter!(
            "editor",
            "ui.workbench_projection.presentation_patch_damage_count",
            damage.len()
        );
        self.publish_refresh_invalidation_diagnostics();
        true
    }

    pub(super) fn apply_scoped_ui_asset_presentation(
        &mut self,
        view_ids: &[ViewInstanceId],
    ) -> bool {
        let mut damage = Vec::new();
        let mut work = ScopedPresentationWork::default();
        for view_id in view_ids {
            let Ok(pane_presentation) = self
                .editor_manager
                .ui_asset_editor_pane_presentation(view_id)
            else {
                work.projection_missing_count += 1;
                record_scoped_presentation_work(&work);
                return false;
            };
            let ui_asset = build_ui_asset_presentation_patch(pane_presentation, &view_id.0);
            let root_patch = patch_ui_asset_presentation(&self.ui, &view_id.0, &ui_asset);
            work.floating_window_rows_visited += root_patch.floating_window_rows_visited;
            work.floating_window_rows_cloned += root_patch.floating_window_rows_cloned;
            if !root_patch.matched_presentation {
                work.projection_missing_count += 1;
                record_scoped_presentation_work(&work);
                return false;
            }
            let patched_native_presenter_ids =
                self.native_window_presenters.patch_ui_asset_presentation(
                    &root_patch.expected_native_presenter_ids,
                    &view_id.0,
                    &ui_asset,
                );
            work.native_presenter_visit_count += patched_native_presenter_ids.presenter_visit_count;
            work.floating_window_rows_visited +=
                patched_native_presenter_ids.floating_window_rows_visited;
            work.floating_window_rows_cloned +=
                patched_native_presenter_ids.floating_window_rows_cloned;
            work.damage_region_count +=
                root_patch.damage.len() + patched_native_presenter_ids.damage_region_count;
            if !scoped_patch_covers_all_presenters(
                &root_patch.damage,
                &root_patch.expected_native_presenter_ids,
                &patched_native_presenter_ids.presenter_ids,
            ) {
                work.presenter_coverage_fallback_count += 1;
                record_scoped_presentation_work(&work);
                return false;
            }
            damage.extend(root_patch.damage);
        }
        for frame in damage {
            self.ui.request_frame_update_region(frame);
        }
        record_current_ui_perf_counter(
            UiPerfCounter::ScopedPresentationPatchCount,
            view_ids.len() as f64,
        );
        record_scoped_presentation_work(&work);
        self.invalidate_committed_pane_payloads();
        self.publish_refresh_invalidation_diagnostics();
        true
    }

    fn invalidate_committed_pane_payloads(&mut self) {
        if let Some(committed) = self.committed_shell_state.as_mut() {
            committed.pane_payloads = None;
            committed.retained_shell_presentation = None;
        }
    }

    fn invalidate_committed_shell_presentation(&mut self) {
        if let Some(committed) = self.committed_shell_state.as_mut() {
            committed.retained_shell_presentation = None;
        }
    }

    pub(super) fn apply_recompute_presentation(
        &mut self,
        model: &WorkbenchViewModel,
        chrome: &EditorChromeSnapshot,
        geometry: &WorkbenchShellGeometry,
        pane_payloads: &HostLifecyclePanePayloads,
        componentized_workbench_layout_frames: callback_dispatch::BuiltinWorkbenchWindowLayoutFrames,
        floating_window_projection_bundle: &FloatingWindowProjectionBundle,
        reuse_shell_layout: bool,
    ) -> Option<std::sync::Arc<crate::ui::layouts::windows::workbench_host_window::ShellPresentation>>
    {
        zircon_runtime::profile_scope!("editor", "retained_host", "recompute_apply_presentation");
        let filtered_hierarchy_entries = self.filtered_hierarchy_entries(&chrome.scene_entries);
        let filtered_chrome = filtered_hierarchy_entries.map(|scene_entries| {
            let mut chrome = chrome.clone();
            chrome.scene_entries = scene_entries;
            chrome
        });
        let chrome = filtered_chrome.as_ref().unwrap_or(chrome);
        let has_component_showcase_runtime =
            self.prepare_component_showcase_runtime_for_presentation(model);
        let pane_template_runtime = if has_component_showcase_runtime {
            &self.component_showcase_runtime
        } else {
            self.builtin_template_runtime.as_ref()
        };
        let hierarchy_filter_query = self.hierarchy_filter_query.clone();
        apply_presentation_with_template_v2_data(
            &self.ui,
            model,
            chrome,
            geometry,
            &pane_payloads.preset_names,
            self.active_layout_preset.as_deref(),
            &pane_payloads.ui_asset_panes,
            &pane_payloads.animation_panes,
            Some(&pane_payloads.runtime_diagnostics),
            &pane_payloads.module_plugins,
            &pane_payloads.build_export,
            &pane_payloads.template_v2_data,
            Some(self.template_bridge.host_projection()),
            Some(self.workbench_window_bridge.host_projection()),
            componentized_workbench_layout_frames,
            floating_window_projection_bundle,
            Some(pane_template_runtime),
            self.template_bridge.presentation_scale_factor(),
            &hierarchy_filter_query,
            &mut self.host_chrome_projection_cache,
            &mut self.console_pane_projection_cache,
            &mut self.module_plugins_pane_projection_cache,
            reuse_shell_layout,
        )
    }
}

fn shell_content_requires_hierarchy_filter(
    content_kind: crate::ui::workbench::snapshot::ViewContentKind,
) -> bool {
    content_kind == crate::ui::workbench::snapshot::ViewContentKind::Hierarchy
}

fn shell_content_requires_component_runtime(
    content_kind: crate::ui::workbench::snapshot::ViewContentKind,
) -> bool {
    content_kind == crate::ui::workbench::snapshot::ViewContentKind::UiComponentShowcase
}

fn workbench_projection_damage(
    previous: &crate::ui::retained_host::primitives::ModelRc<TemplatePaneNodeData>,
    next: &crate::ui::retained_host::primitives::ModelRc<TemplatePaneNodeData>,
    changed_rows: &[usize],
) -> Vec<FrameRect> {
    changed_rows
        .iter()
        .flat_map(|row| [previous.get(*row), next.get(*row)])
        .flatten()
        .filter_map(|node| {
            (node.frame.width > 0.0 && node.frame.height > 0.0).then_some(FrameRect {
                x: node.frame.x,
                y: node.frame.y,
                width: node.frame.width,
                height: node.frame.height,
            })
        })
        .collect()
}

#[derive(Clone, Copy)]
enum WorkbenchProjectionFallback {
    FullProjection,
    Projection,
    HitIndex,
}

fn record_workbench_projection_fallback(reason: WorkbenchProjectionFallback) {
    zircon_runtime::profile_counter!(
        "editor",
        "ui.workbench_projection.presentation_patch_fallback_count",
        1
    );
    let counter = match reason {
        WorkbenchProjectionFallback::FullProjection => {
            "ui.workbench_projection.presentation_patch_fallback_full_projection_count"
        }
        WorkbenchProjectionFallback::Projection => {
            "ui.workbench_projection.presentation_patch_fallback_projection_count"
        }
        WorkbenchProjectionFallback::HitIndex => {
            "ui.workbench_projection.presentation_patch_fallback_hit_index_count"
        }
    };
    zircon_runtime::profile_counter!("editor", counter, 1);
}

#[derive(Default)]
struct ScopedPresentationWork {
    floating_window_rows_visited: usize,
    floating_window_rows_cloned: usize,
    native_presenter_visit_count: usize,
    damage_region_count: usize,
    projection_missing_count: usize,
    presenter_coverage_fallback_count: usize,
}

fn record_scoped_presentation_work(work: &ScopedPresentationWork) {
    record_current_ui_perf_counter(
        UiPerfCounter::ScopedPresentationFloatingWindowRowsVisited,
        work.floating_window_rows_visited as f64,
    );
    record_current_ui_perf_counter(
        UiPerfCounter::ScopedPresentationFloatingWindowRowsCloned,
        work.floating_window_rows_cloned as f64,
    );
    record_current_ui_perf_counter(
        UiPerfCounter::ScopedPresentationNativePresenterVisitCount,
        work.native_presenter_visit_count as f64,
    );
    record_current_ui_perf_counter(
        UiPerfCounter::ScopedPresentationDamageRegionCount,
        work.damage_region_count as f64,
    );
    record_current_ui_perf_counter(
        UiPerfCounter::ScopedPresentationProjectionMissingCount,
        work.projection_missing_count as f64,
    );
    record_current_ui_perf_counter(
        UiPerfCounter::ScopedPresentationPresenterCoverageFallbackCount,
        work.presenter_coverage_fallback_count as f64,
    );
}

fn scoped_patch_covers_all_presenters(
    root_damage: &[crate::ui::retained_host::host_contract::FrameRect],
    expected_native_presenter_ids: &BTreeSet<crate::ui::workbench::layout::MainPageId>,
    patched_native_presenter_ids: &BTreeSet<crate::ui::workbench::layout::MainPageId>,
) -> bool {
    (expected_native_presenter_ids == patched_native_presenter_ids)
        && (!root_damage.is_empty() || !patched_native_presenter_ids.is_empty())
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeSet;

    use crate::ui::layouts::common::model_rc;
    use crate::ui::retained_host::host_contract::{FrameRect, TemplatePaneNodeData};
    use crate::ui::workbench::layout::MainPageId;
    use crate::ui::workbench::snapshot::ViewContentKind;

    use super::{
        scoped_patch_covers_all_presenters, shell_content_requires_component_runtime,
        shell_content_requires_hierarchy_filter, workbench_projection_damage,
    };

    #[test]
    fn early_shell_content_prepares_only_target_specific_support_data() {
        assert!(shell_content_requires_hierarchy_filter(
            ViewContentKind::Hierarchy
        ));
        assert!(!shell_content_requires_hierarchy_filter(
            ViewContentKind::Inspector
        ));

        assert!(shell_content_requires_component_runtime(
            ViewContentKind::UiComponentShowcase
        ));
        assert!(!shell_content_requires_component_runtime(
            ViewContentKind::Hierarchy
        ));
    }

    #[test]
    fn workbench_projection_damage_covers_old_and_new_row_frames() {
        let mut previous_node = TemplatePaneNodeData::default();
        previous_node.frame.x = 12.0;
        previous_node.frame.y = 20.0;
        previous_node.frame.width = 100.0;
        previous_node.frame.height = 24.0;
        let mut next_node = previous_node.clone();
        next_node.frame.x = 32.0;
        let previous = model_rc(vec![previous_node]);
        let next = model_rc(vec![next_node]);

        assert_eq!(
            workbench_projection_damage(&previous, &next, &[0]),
            vec![
                FrameRect {
                    x: 12.0,
                    y: 20.0,
                    width: 100.0,
                    height: 24.0,
                },
                FrameRect {
                    x: 32.0,
                    y: 20.0,
                    width: 100.0,
                    height: 24.0,
                },
            ]
        );
    }

    #[test]
    fn scoped_patch_falls_back_when_a_root_declared_native_presenter_is_not_patched() {
        let root_damage = [FrameRect {
            x: 0.0,
            y: 0.0,
            width: 320.0,
            height: 180.0,
        }];

        let expected = BTreeSet::from([MainPageId::new("window:target")]);
        let missing = BTreeSet::new();

        assert!(!scoped_patch_covers_all_presenters(
            &root_damage,
            &expected,
            &missing
        ));
        assert!(scoped_patch_covers_all_presenters(
            &root_damage,
            &expected,
            &expected
        ));
        assert!(!scoped_patch_covers_all_presenters(
            &root_damage,
            &expected,
            &BTreeSet::from([MainPageId::new("window:other")])
        ));
    }

    #[test]
    fn coverage_fallback_records_damage_before_returning() {
        let source = include_str!("presentation.rs");
        let scoped_patch = source
            .split_once("let patched_native_presenter_ids")
            .and_then(|(_, tail)| tail.split_once("damage.extend(root_patch.damage)"))
            .map(|(_, tail)| tail)
            .expect("scoped patch aggregation should remain in the recompute fast path");
        let damage_count = scoped_patch
            .find("work.damage_region_count +=")
            .expect("damage should be counted");
        let coverage_fallback = scoped_patch
            .find("if !scoped_patch_covers_all_presenters")
            .expect("coverage should be checked");

        assert!(
            damage_count < coverage_fallback,
            "damage produced before a coverage fallback must remain observable"
        );
    }

    #[test]
    fn missing_root_projection_records_probe_work_before_returning() {
        let source = include_str!("presentation.rs");
        let scoped_patch = source
            .split_once("let root_patch = patch_ui_asset_presentation")
            .and_then(|(_, tail)| tail.split_once("let patched_native_presenter_ids"))
            .map(|(_, tail)| tail)
            .expect("root patch work should remain in the scoped recompute path");
        let root_rows = scoped_patch
            .find("work.floating_window_rows_visited += root_patch")
            .expect("root probe work should be counted");
        let projection_missing = scoped_patch
            .find("if !root_patch.matched_presentation")
            .expect("root projection failure should fall back");

        assert!(
            root_rows < projection_missing,
            "a root projection fallback must retain the probe work already performed"
        );
    }
}
