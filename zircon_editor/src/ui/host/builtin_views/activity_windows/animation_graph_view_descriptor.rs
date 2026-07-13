use crate::core::commands::DocumentKind;
use crate::ui::workbench::autolayout::default_constraints_for_content;
use crate::ui::workbench::snapshot::ViewContentKind;
use crate::ui::workbench::view::{
    PaneBodySpec, PaneInteractionMode, PanePayloadKind, PaneRouteNamespace, PaneTemplateSpec,
    PreferredHost, ViewDescriptor, ViewDescriptorId, ViewKind,
};

pub(super) fn animation_graph_view_descriptor() -> ViewDescriptor {
    ViewDescriptor::new(
        ViewDescriptorId::new("editor.animation_graph"),
        ViewKind::ActivityWindow,
        "Animation Graph",
    )
    .with_document_kind(DocumentKind::animation_graph())
    .with_multi_instance(true)
    .with_preferred_host(PreferredHost::DocumentCenter)
    .with_default_constraints(default_constraints_for_content(
        ViewContentKind::AnimationGraphEditor,
    ))
    .with_pane_template(PaneTemplateSpec::new(PaneBodySpec::new(
        "res://ui/editor/host/animation_graph_body.zui",
        PanePayloadKind::AnimationGraphV1,
        PaneRouteNamespace::Animation,
        PaneInteractionMode::HybridNativeSlot,
    )))
    .with_icon_key("animation-graph")
}
