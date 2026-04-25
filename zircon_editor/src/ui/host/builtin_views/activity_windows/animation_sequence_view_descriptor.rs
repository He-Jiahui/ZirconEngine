use crate::ui::workbench::autolayout::default_constraints_for_content;
use crate::ui::workbench::snapshot::ViewContentKind;
use crate::ui::workbench::view::{
    PaneBodySpec, PaneInteractionMode, PanePayloadKind, PaneRouteNamespace, PaneTemplateSpec,
    PreferredHost, ViewDescriptor, ViewDescriptorId, ViewKind,
};

pub(super) fn animation_sequence_view_descriptor() -> ViewDescriptor {
    ViewDescriptor::new(
        ViewDescriptorId::new("editor.animation_sequence"),
        ViewKind::ActivityWindow,
        "Animation Sequence",
    )
    .with_multi_instance(true)
    .with_preferred_host(PreferredHost::DocumentCenter)
    .with_default_constraints(default_constraints_for_content(
        ViewContentKind::AnimationSequenceEditor,
    ))
    .with_pane_template(PaneTemplateSpec::new(PaneBodySpec::new(
        "pane.animation.sequence.body",
        PanePayloadKind::AnimationSequenceV1,
        PaneRouteNamespace::Animation,
        PaneInteractionMode::HybridNativeSlot,
    )))
    .with_icon_key("animation-sequence")
}
