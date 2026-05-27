use super::shared::*;

pub(super) fn descriptors() -> Vec<UiComponentDescriptor> {
    vec![
        editor_panel_layout(
            "ViewportHost",
            "Viewport Host",
            UiComponentLayoutRole::Canvas,
            "viewport-host",
        )
        .with_prop(string_prop("camera_target"))
        .slot(UiSlotSchema::new("overlays").multiple(true))
        .events([
            UiComponentEventKind::SetWorldSurface,
            UiComponentEventKind::SetWorldTransform,
        ])
        .requires_host_capability(UiHostCapability::CanvasRender)
        .requires_render_capability(UiRenderCapability::Canvas),
        editor_panel_layout(
            "GraphCanvas",
            "Graph Canvas",
            UiComponentLayoutRole::Canvas,
            "graph-canvas",
        )
        .slot(UiSlotSchema::new("nodes").multiple(true))
        .slot(UiSlotSchema::new("edges").multiple(true))
        .events([
            UiComponentEventKind::SelectOption,
            UiComponentEventKind::BeginDrag,
            UiComponentEventKind::EndDrag,
            UiComponentEventKind::DropHover,
        ])
        .requires_host_capability(UiHostCapability::CanvasRender)
        .requires_render_capability(UiRenderCapability::Canvas),
        editor_panel_component(
            "Timeline",
            "Timeline",
            UiComponentCategory::Numeric,
            "timeline",
        )
        .with_prop(default_string_prop("component", "ul"))
        .with_prop(enum_prop_with_options(
            "position",
            "right",
            ["alternate-reverse", "alternate", "left", "right"]
                .into_iter()
                .map(enum_option_descriptor),
        ))
        .with_prop(float_prop("time", 0.0))
        .with_prop(float_prop("duration", 0.0))
        .slot(UiSlotSchema::new("items").multiple(true))
        .slot(UiSlotSchema::new("content").multiple(true))
        .events([
            UiComponentEventKind::ValueChanged,
            UiComponentEventKind::DragDelta,
        ]),
        editor_panel_layout(
            "VisualDesigner",
            "Visual Designer",
            UiComponentLayoutRole::Canvas,
            "visual-designer",
        )
        .slot(UiSlotSchema::new("content").multiple(true))
        .slot(UiSlotSchema::new("overlays").multiple(true))
        .events([
            UiComponentEventKind::SelectOption,
            UiComponentEventKind::BeginDrag,
            UiComponentEventKind::EndDrag,
            UiComponentEventKind::DropHover,
        ])
        .requires_host_capability(UiHostCapability::CanvasRender)
        .requires_render_capability(UiRenderCapability::Canvas),
        shell("View", "View", "view")
            .with_prop(required_string_prop("view_id"))
            .with_prop(text_prop())
            .with_prop(bool_prop("dirty", false))
            .slot(UiSlotSchema::new("content").multiple(true))
            .event(UiComponentEventKind::Focus),
        shell("Window", "Window", "window")
            .with_prop(required_string_prop("window_id"))
            .with_prop(text_prop())
            .with_prop(enum_prop("dock_policy", "main_workbench"))
            .with_prop(bool_prop("floating", false))
            .slot(UiSlotSchema::new("views").multiple(true))
            .events([
                UiComponentEventKind::Focus,
                UiComponentEventKind::BeginDrag,
                UiComponentEventKind::EndDrag,
            ]),
        shell("WindowFrame", "Window Frame", "window-frame")
            .with_prop(text_prop())
            .slot(UiSlotSchema::new("chrome"))
            .slot(UiSlotSchema::new("content").multiple(true))
            .event(UiComponentEventKind::ClosePopup),
        shell("DocumentNode", "Document Node", "document-node")
            .descriptor_kind(UiComponentDescriptorKind::Layout)
            .layout_role(UiComponentLayoutRole::EditorDock)
            .with_prop(enum_prop("node_kind", "tabs"))
            .slot(UiSlotSchema::new("content").multiple(true)),
        shell("FloatingWindow", "Floating Window", "floating-window")
            .with_prop(required_string_prop("window_id"))
            .with_prop(string_prop("focused_view"))
            .slot(UiSlotSchema::new("content").multiple(true))
            .events([
                UiComponentEventKind::Focus,
                UiComponentEventKind::BeginDrag,
                UiComponentEventKind::EndDrag,
            ]),
        shell("DockHost", "Dock Host", "dock-host")
            .descriptor_kind(UiComponentDescriptorKind::Layout)
            .layout_role(UiComponentLayoutRole::EditorDock)
            .with_prop(string_prop("active_window"))
            .slot(UiSlotSchema::new("windows").multiple(true))
            .events([UiComponentEventKind::Focus, UiComponentEventKind::DropHover]),
        shell("WorkbenchShell", "Workbench Shell", "workbench-shell")
            .with_prop(workbench_skin_prop())
            .with_prop(fyrox_panel_preset_prop())
            .with_prop(jetbrains_shell_preset_prop())
            .with_prop(unreal_window_model_preset_prop())
            .with_prop(string_prop("active_window"))
            .slot(UiSlotSchema::new("drawers").multiple(true))
            .slot(UiSlotSchema::new("documents").multiple(true))
            .slot(UiSlotSchema::new("status"))
            .slot(UiSlotSchema::new("menu"))
            .event(UiComponentEventKind::Commit),
    ]
}
