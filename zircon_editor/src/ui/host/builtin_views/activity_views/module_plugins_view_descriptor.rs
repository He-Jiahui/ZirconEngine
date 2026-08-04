use crate::ui::workbench::autolayout::default_constraints_for_content;
use crate::ui::workbench::layout::ActivityDrawerSlot;
use crate::ui::workbench::snapshot::ViewContentKind;
use crate::ui::workbench::view::WorkbenchSlot;
use crate::ui::workbench::view::{
    PaneBodySpec, PaneInteractionMode, PanePayloadKind, PaneRouteNamespace, PaneTemplateSpec,
    ViewDescriptor, ViewDescriptorId, ViewKind,
};

pub(super) fn module_plugins_view_descriptor() -> ViewDescriptor {
    ViewDescriptor::new(
        ViewDescriptorId::new("editor.module_plugins"),
        ViewKind::ActivityView,
        "Plugin Manager",
    )
    .with_workbench_slot(WorkbenchSlot::LeftBottomDrawer)
    .with_default_constraints(default_constraints_for_content(
        ViewContentKind::ModulePlugins,
    ))
    .with_pane_template(PaneTemplateSpec::new(PaneBodySpec::new(
        "res://ui/editor/host/module_plugins_body.zui",
        PanePayloadKind::ModulePluginsV1,
        PaneRouteNamespace::Dock,
        PaneInteractionMode::HybridNativeSlot,
    )))
    .with_icon_key("module-plugins")
}
