use crate::ui::workbench::snapshot::ViewContentKind;
use crate::ui::workbench::view::{
    ActivityWindowTemplateSpec, ViewDescriptor, ViewDescriptorId, ViewKind, WorkbenchSlot,
};

pub(super) fn workbench_window_view_descriptor() -> ViewDescriptor {
    ViewDescriptor::new(
        ViewDescriptorId::new("editor.workbench_window"),
        ViewKind::ActivityWindow,
        "Workbench",
    )
    .with_workbench_slot(WorkbenchSlot::DocumentCenter)
    .with_default_constraints(
        crate::ui::workbench::autolayout::default_constraints_for_content(ViewContentKind::Scene),
    )
    .with_activity_window_template(ActivityWindowTemplateSpec::new(
        "res://ui/editor/windows/workbench_window.zui",
    ))
    .with_icon_key("workbench")
}
