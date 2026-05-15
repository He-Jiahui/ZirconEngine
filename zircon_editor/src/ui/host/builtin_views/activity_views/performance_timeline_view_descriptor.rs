use crate::ui::workbench::autolayout::default_constraints_for_content;
use crate::ui::workbench::layout::ActivityDrawerSlot;
use crate::ui::workbench::snapshot::ViewContentKind;
use crate::ui::workbench::view::{
    PaneBodySpec, PaneInteractionMode, PanePayloadKind, PaneRouteNamespace, PaneTemplateSpec,
    ViewDescriptor, ViewDescriptorId, ViewKind,
};

pub(super) fn performance_timeline_view_descriptor() -> ViewDescriptor {
    ViewDescriptor::new(
        ViewDescriptorId::new("editor.performance_timeline"),
        ViewKind::ActivityView,
        "Performance Timeline",
    )
    .with_preferred_drawer_slot(ActivityDrawerSlot::Bottom)
    .with_default_constraints(default_constraints_for_content(
        ViewContentKind::PerformanceTimeline,
    ))
    .with_pane_template(PaneTemplateSpec::new(PaneBodySpec::new(
        "pane.performance.timeline.body",
        PanePayloadKind::PerformanceTimelineV1,
        PaneRouteNamespace::Diagnostics,
        PaneInteractionMode::HybridNativeSlot,
    )))
    .with_icon_key("performance-timeline")
}
