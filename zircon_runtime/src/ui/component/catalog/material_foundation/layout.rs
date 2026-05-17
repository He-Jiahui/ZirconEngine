use super::shared::*;

pub(super) fn descriptors() -> Vec<UiComponentDescriptor> {
    vec![
        layout("Box", "Box", UiComponentLayoutRole::Flex, "box"),
        layout(
            "Container",
            "Container",
            UiComponentLayoutRole::Flex,
            "container",
        )
        .with_prop(float_prop("max_width", 1200.0)),
        layout("Grid", "Grid", UiComponentLayoutRole::Grid, "grid")
            .with_prop(int_prop("columns", 12)),
        layout("Stack", "Stack", UiComponentLayoutRole::Flex, "stack")
            .with_prop(enum_prop("direction", "column")),
        layout("Masonry", "Masonry", UiComponentLayoutRole::Grid, "masonry")
            .with_prop(int_prop("columns", 3))
            .with_prop(bool_prop("needs_support", true)),
        primitive(
            "Scrollbar",
            "Scrollbar",
            UiComponentCategory::Numeric,
            "scrollbar",
        )
        .with_prop(number_value_prop())
        .event(UiComponentEventKind::ValueChanged)
        .requires_render_capability(UiRenderCapability::Scroll),
        layout(
            "Splitter",
            "Splitter",
            UiComponentLayoutRole::Size,
            "splitter",
        )
        .with_prop(number_value_prop())
        .event(UiComponentEventKind::ValueChanged),
        composite("Slot", "Slot", UiComponentCategory::Container, "slot")
            .with_prop(
                UiPropSchema::new("name", UiValueKind::String)
                    .default_value(UiValue::String("content".to_string())),
            )
            .slot(UiSlotSchema::new("content").multiple(true)),
        composite(
            "Composite",
            "Composite",
            UiComponentCategory::Container,
            "composite",
        )
        .slot(UiSlotSchema::new("content").multiple(true)),
        layout(
            "FlexGroup",
            "Flex Group",
            UiComponentLayoutRole::Flex,
            "flex-group",
        ),
        layout(
            "HorizontalGroup",
            "Horizontal Group",
            UiComponentLayoutRole::Flex,
            "horizontal-group",
        ),
        layout(
            "VerticalGroup",
            "Vertical Group",
            UiComponentLayoutRole::Flex,
            "vertical-group",
        ),
        layout(
            "GridGroup",
            "Grid Group",
            UiComponentLayoutRole::Grid,
            "grid-group",
        ),
        layout(
            "Overlay",
            "Overlay",
            UiComponentLayoutRole::Overlay,
            "overlay",
        ),
        layout(
            "ScrollView",
            "Scroll View",
            UiComponentLayoutRole::Flex,
            "scroll-view",
        )
        .requires_render_capability(UiRenderCapability::Scroll),
        composite(
            "ClickAwayListener",
            "Click Away Listener",
            UiComponentCategory::Container,
            "click-away-listener",
        )
        .with_prop(bool_prop("behavior_utility", true))
        .slot(UiSlotSchema::new("content").multiple(true))
        .event(UiComponentEventKind::ClosePopup),
        composite("Portal", "Portal", UiComponentCategory::Container, "portal")
            .with_prop(bool_prop("behavior_utility", true))
            .slot(UiSlotSchema::new("content").multiple(true)),
        primitive("NoSsr", "No Ssr", UiComponentCategory::Container, "no-ssr")
            .with_prop(bool_prop("behavior_utility", true)),
        primitive(
            "CssBaseline",
            "Css Baseline",
            UiComponentCategory::Container,
            "css-baseline",
        )
        .with_prop(bool_prop("behavior_utility", true)),
        primitive(
            "InitColorSchemeScript",
            "Init Color Scheme Script",
            UiComponentCategory::Container,
            "init-color-scheme-script",
        )
        .with_prop(bool_prop("behavior_utility", true)),
        primitive(
            "UseMediaQuery",
            "Use Media Query",
            UiComponentCategory::Container,
            "use-media-query",
        )
        .with_prop(string_prop("query"))
        .with_prop(bool_prop("behavior_utility", true)),
        transition("Collapse", "Collapse", "collapse"),
        transition("Fade", "Fade", "fade"),
        transition("Grow", "Grow", "grow"),
        transition("Slide", "Slide", "slide"),
        transition("Zoom", "Zoom", "zoom"),
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
        .with_prop(float_prop("time", 0.0))
        .with_prop(float_prop("duration", 0.0))
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

fn transition(id: &str, display_name: &str, role: &str) -> UiComponentDescriptor {
    composite(id, display_name, UiComponentCategory::Container, role)
        .with_prop(bool_prop("in", true))
        .with_prop(int_prop("timeout_ms", 150))
        .slot(UiSlotSchema::new("content").multiple(true))
}
