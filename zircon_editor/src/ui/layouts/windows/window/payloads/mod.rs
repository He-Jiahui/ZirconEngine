#[path = "animation_graph.rs"]
mod animation_graph;
#[path = "animation_sequence.rs"]
mod animation_sequence;
#[path = "build_export.rs"]
mod build_export;
#[path = "component_showcase.rs"]
mod component_showcase;
#[path = "console.rs"]
mod console;
#[path = "generated_bottom.rs"]
mod generated_bottom;
#[path = "hierarchy.rs"]
mod hierarchy;
#[path = "inspector.rs"]
mod inspector;
#[path = "module_plugins.rs"]
mod module_plugins;
#[path = "performance_timeline.rs"]
mod performance_timeline;
#[path = "runtime_diagnostics.rs"]
mod runtime_diagnostics;

use crate::ui::workbench::view::PanePayloadKind;

use super::pane_payload::{PanePayload, TemplateV2PanePayload};
use super::pane_presentation::PanePayloadBuildContext;

pub(super) fn build_payload(
    payload_kind: PanePayloadKind,
    context: &PanePayloadBuildContext<'_>,
) -> PanePayload {
    match payload_kind {
        PanePayloadKind::TemplateV2 => PanePayload::TemplateV2(TemplateV2PanePayload {
            values: context
                .template_v2_snapshot
                .map(|snapshot| snapshot.values().clone())
                .unwrap_or_default(),
            component_patches: context
                .template_v2_snapshot
                .map(|snapshot| snapshot.component_patches().to_vec())
                .unwrap_or_default(),
        }),
        PanePayloadKind::ConsoleV1 => console::build(context),
        PanePayloadKind::InspectorV1 => inspector::build(context),
        PanePayloadKind::HierarchyV1 => hierarchy::build(context),
        PanePayloadKind::AnimationSequenceV1 => animation_sequence::build(context),
        PanePayloadKind::AnimationGraphV1 => animation_graph::build(context),
        PanePayloadKind::RuntimeDiagnosticsV1 => runtime_diagnostics::build(context),
        PanePayloadKind::PerformanceTimelineV1 => performance_timeline::build(context),
        PanePayloadKind::ModulePluginsV1 => module_plugins::build(context),
        PanePayloadKind::BuildExportV1 => build_export::build(context),
        PanePayloadKind::GeneratedBottomV1 => generated_bottom::build(context),
        PanePayloadKind::UiComponentShowcaseV1 => component_showcase::build(context),
    }
}
