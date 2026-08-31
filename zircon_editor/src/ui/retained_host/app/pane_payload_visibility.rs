use crate::ui::workbench::model::WorkbenchViewModel;
use crate::ui::workbench::snapshot::ViewContentKind;

pub(super) fn should_collect_payload_for_kind(
    model: &WorkbenchViewModel,
    kind: ViewContentKind,
) -> bool {
    payload_visibility_for_pair(model, kind, kind).0
}

pub(super) fn payload_visibility_for_pair(
    model: &WorkbenchViewModel,
    first: ViewContentKind,
    second: ViewContentKind,
) -> (bool, bool) {
    let mut visible = (false, false);

    for tab in &model.document_tabs {
        if tab.active {
            record_visible_kind(&mut visible, tab.content_kind, first, second);
        }
        if visible.0 && visible.1 {
            return visible;
        }
    }

    for stack in model.tool_windows.values() {
        if !stack.visible {
            continue;
        }
        for tab in &stack.tabs {
            if tab.active || stack.active_tab.as_ref() == Some(&tab.instance_id) {
                record_visible_kind(&mut visible, tab.content_kind, first, second);
            }
            if visible.0 && visible.1 {
                return visible;
            }
        }
    }

    for window in &model.floating_windows {
        for tab in &window.tabs {
            if tab.active {
                record_visible_kind(&mut visible, tab.content_kind, first, second);
            }
            if visible.0 && visible.1 {
                return visible;
            }
        }
    }

    visible
}

#[inline]
fn record_visible_kind(
    visible: &mut (bool, bool),
    kind: ViewContentKind,
    first: ViewContentKind,
    second: ViewContentKind,
) {
    visible.0 |= kind == first;
    visible.1 |= kind == second;
}

#[cfg(test)]
mod paired_visibility_tests {
    use std::collections::BTreeMap;
    use std::hint::black_box;
    use std::time::Instant;

    use super::*;
    use crate::core::commands::{EditorKeymap, MenuBarModel};
    use crate::ui::workbench::autolayout::ShellFrame;
    use crate::ui::workbench::layout::{
        ActivityDrawerMode, ActivityDrawerSlot, MainPageId, WorkspaceTarget,
    };
    use crate::ui::workbench::model::{
        DocumentTabModel, DocumentWorkspaceModel, DrawerRingModel, FloatingWindowModel,
        MainHostStripModel, MainHostStripViewModel, PaneTabModel, StatusBarModel,
        ToolWindowStackModel,
    };
    use crate::ui::workbench::snapshot::DocumentWorkspaceSnapshot;
    use crate::ui::workbench::view::{ViewDescriptorId, ViewInstanceId};

    const BENCHMARK_MARKER: &str = "EDITOR264_PAIRED_PANE_PAYLOAD_VISIBILITY_BENCH_V1";

    #[test]
    fn optimization_batch_20260829as_paired_visibility_matches_two_legacy_queries() {
        let model = model_with_tabs(vec![
            document_tab("inactive-module", ViewContentKind::ModulePlugins, false),
            document_tab("active-build", ViewContentKind::BuildExport, true),
            document_tab("active-module", ViewContentKind::ModulePlugins, true),
        ]);

        let paired = payload_visibility_for_pair(
            &model,
            ViewContentKind::ModulePlugins,
            ViewContentKind::BuildExport,
        );

        assert_eq!(
            paired,
            (
                legacy_payload_visibility(&model, ViewContentKind::ModulePlugins),
                legacy_payload_visibility(&model, ViewContentKind::BuildExport),
            )
        );
        assert_eq!(paired, (true, true));
    }

    #[test]
    fn optimization_batch_20260829as_paired_visibility_honors_selected_tool_and_floating_tabs() {
        let mut model = model_with_tabs(vec![
            document_tab("inactive-module", ViewContentKind::ModulePlugins, false),
            document_tab("inactive-build", ViewContentKind::BuildExport, false),
        ]);
        model.tool_windows.insert(
            ActivityDrawerSlot::LeftTop,
            ToolWindowStackModel {
                slot: ActivityDrawerSlot::LeftTop,
                mode: ActivityDrawerMode::Pinned,
                visible: true,
                tabs: vec![pane_tab(
                    "selected-module",
                    ViewContentKind::ModulePlugins,
                    false,
                )],
                active_tab: Some(ViewInstanceId::new("selected-module")),
            },
        );
        model.floating_windows.push(FloatingWindowModel {
            window_id: MainPageId::new("window:build-export"),
            title: "Build Export".to_owned(),
            requested_frame: ShellFrame::default(),
            focused_view: None,
            tabs: vec![document_tab(
                "active-build",
                ViewContentKind::BuildExport,
                true,
            )],
        });

        let paired = payload_visibility_for_pair(
            &model,
            ViewContentKind::ModulePlugins,
            ViewContentKind::BuildExport,
        );
        assert_eq!(paired, (true, true));
        assert_eq!(
            paired,
            (
                legacy_payload_visibility(&model, ViewContentKind::ModulePlugins),
                legacy_payload_visibility(&model, ViewContentKind::BuildExport),
            )
        );
    }

    #[test]
    #[ignore = "release-only performance gate"]
    fn optimization_batch_20260829as_paired_visibility_meets_release_performance_gate() {
        let model = model_with_tabs(
            (0..4_096)
                .map(|index| document_tab(&format!("scene-{index}"), ViewContentKind::Scene, true))
                .collect(),
        );
        let mut baseline_samples = Vec::with_capacity(31);
        let mut candidate_samples = Vec::with_capacity(31);

        for sample in 0..31 {
            let run_baseline = || {
                let started = Instant::now();
                for _ in 0..32 {
                    black_box(legacy_payload_visibility(
                        black_box(&model),
                        black_box(ViewContentKind::ModulePlugins),
                    ));
                    black_box(legacy_payload_visibility(
                        black_box(&model),
                        black_box(ViewContentKind::BuildExport),
                    ));
                }
                started.elapsed().as_nanos()
            };
            let run_candidate = || {
                let started = Instant::now();
                for _ in 0..32 {
                    black_box(payload_visibility_for_pair(
                        black_box(&model),
                        black_box(ViewContentKind::ModulePlugins),
                        black_box(ViewContentKind::BuildExport),
                    ));
                }
                started.elapsed().as_nanos()
            };

            if sample % 2 == 0 {
                baseline_samples.push(run_baseline());
                candidate_samples.push(run_candidate());
            } else {
                candidate_samples.push(run_candidate());
                baseline_samples.push(run_baseline());
            }
        }

        baseline_samples.sort_unstable();
        candidate_samples.sort_unstable();
        let baseline_p95_ns = baseline_samples[29];
        let candidate_p95_ns = candidate_samples[29];
        let ratio = candidate_p95_ns as f64 / baseline_p95_ns as f64;
        eprintln!(
            "{BENCHMARK_MARKER} baseline_p95_ns={baseline_p95_ns} candidate_p95_ns={candidate_p95_ns} ratio={ratio:.4}"
        );
        assert!(
            candidate_p95_ns * 100 <= baseline_p95_ns * 70,
            "paired pane payload visibility ratio {ratio:.4} exceeded 0.70"
        );
    }

    fn legacy_payload_visibility(model: &WorkbenchViewModel, kind: ViewContentKind) -> bool {
        model
            .document_tabs
            .iter()
            .any(|tab| tab.active && tab.content_kind == kind)
            || model.tool_windows.values().any(|stack| {
                stack.visible
                    && stack.tabs.iter().any(|tab| {
                        (tab.active || stack.active_tab.as_ref() == Some(&tab.instance_id))
                            && tab.content_kind == kind
                    })
            })
            || model.floating_windows.iter().any(|window| {
                window
                    .tabs
                    .iter()
                    .any(|tab| tab.active && tab.content_kind == kind)
            })
    }

    fn document_tab(id: &str, content_kind: ViewContentKind, active: bool) -> DocumentTabModel {
        DocumentTabModel {
            workspace: WorkspaceTarget::MainPage(MainPageId::workbench()),
            workspace_path: Vec::new(),
            instance_id: ViewInstanceId::new(id),
            descriptor_id: ViewDescriptorId::new(id),
            title: id.to_owned(),
            icon_key: "tool".to_owned(),
            content_kind,
            active,
            closeable: true,
            empty_state: None,
        }
    }

    fn pane_tab(id: &str, content_kind: ViewContentKind, active: bool) -> PaneTabModel {
        PaneTabModel {
            instance_id: ViewInstanceId::new(id),
            descriptor_id: ViewDescriptorId::new(id),
            title: id.to_owned(),
            icon_key: "tool".to_owned(),
            content_kind,
            active,
            closeable: false,
            empty_state: None,
        }
    }

    fn model_with_tabs(document_tabs: Vec<DocumentTabModel>) -> WorkbenchViewModel {
        WorkbenchViewModel {
            is_playing: false,
            asset_creation_menu: Default::default(),
            keymap: EditorKeymap::default_workbench(),
            menu_bar: MenuBarModel { menus: Vec::new() },
            host_strip: MainHostStripViewModel {
                mode: MainHostStripModel::Workbench,
                pages: Vec::new(),
                active_page: MainPageId::workbench(),
                breadcrumbs: Vec::new(),
            },
            drawer_ring: DrawerRingModel {
                visible: false,
                drawers: BTreeMap::new(),
            },
            tool_windows: BTreeMap::new(),
            document_tabs,
            floating_windows: Vec::new(),
            document: DocumentWorkspaceModel::Workbench {
                page_id: MainPageId::workbench(),
                title: "Workbench".to_owned(),
                workspace: DocumentWorkspaceSnapshot::Tabs {
                    tabs: Vec::new(),
                    active_tab: None,
                },
            },
            status_bar: StatusBarModel::default(),
        }
    }
}
