use crate::core::commands::DocumentKind;
use crate::ui::workbench::autolayout::default_constraints_for_content;
use crate::ui::workbench::snapshot::ViewContentKind;
use crate::ui::workbench::view::{
    PaneBodySpec, PaneInteractionMode, PanePayloadKind, PaneRouteNamespace, PaneTemplateSpec,
    ViewDescriptor, ViewDescriptorId, ViewKind, WorkbenchSlot,
};

pub(super) fn animation_sequence_view_descriptor() -> ViewDescriptor {
    ViewDescriptor::new(
        ViewDescriptorId::new("editor.animation_sequence"),
        ViewKind::ActivityWindow,
        "Animation Sequence",
    )
    .with_document_kind(DocumentKind::animation_sequence())
    .with_multi_instance(true)
    .with_workbench_slot(WorkbenchSlot::DocumentCenter)
    .with_default_constraints(default_constraints_for_content(
        ViewContentKind::AnimationSequenceEditor,
    ))
    .with_pane_template(PaneTemplateSpec::new(PaneBodySpec::new(
        "res://ui/editor/host/animation_sequence_body.zui",
        PanePayloadKind::AnimationSequenceV1,
        PaneRouteNamespace::Animation,
        PaneInteractionMode::HybridNativeSlot,
    )))
    .with_icon_key("animation-sequence")
}
