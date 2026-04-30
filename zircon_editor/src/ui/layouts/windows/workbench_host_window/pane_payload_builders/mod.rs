mod animation_graph;
mod animation_sequence;
mod component_showcase;
mod console;
mod hierarchy;
mod inspector;
mod module_plugins;
mod runtime_diagnostics;

use crate::ui::workbench::view::PanePayloadKind;

use super::pane_payload::PanePayload;
use super::pane_presentation::PanePayloadBuildContext;

pub(super) fn build_payload(
    payload_kind: PanePayloadKind,
    context: &PanePayloadBuildContext<'_>,
) -> PanePayload {
    match payload_kind {
        PanePayloadKind::ConsoleV1 => console::build(context),
        PanePayloadKind::InspectorV1 => inspector::build(context),
        PanePayloadKind::HierarchyV1 => hierarchy::build(context),
        PanePayloadKind::AnimationSequenceV1 => animation_sequence::build(context),
        PanePayloadKind::AnimationGraphV1 => animation_graph::build(context),
        PanePayloadKind::RuntimeDiagnosticsV1 => runtime_diagnostics::build(context),
        PanePayloadKind::ModulePluginsV1 => module_plugins::build(context),
        PanePayloadKind::UiComponentShowcaseV1 => component_showcase::build(context),
    }
}
