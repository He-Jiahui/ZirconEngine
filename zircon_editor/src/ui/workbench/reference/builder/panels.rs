use zircon_runtime_interface::ui::{
    event_ui::UiNodeId,
    layout::{UiContainerKind, UiGridBoxConfig, UiLinearBoxConfig},
    tree::UiTreeError,
    widget::UiWidgetBehavior,
};

use super::nodes::{fixed_height_box, fixed_size_box, fixed_width_box, spacer_node, stretch_box};
use super::{normalized_reference_path_segment, ReferenceSurfaceBuilder};

impl ReferenceSurfaceBuilder {
    pub(super) fn build_activity_rail(&mut self, parent: UiNodeId) -> Result<(), UiTreeError> {
        let rail = self.ids.activity_rail;
        self.insert_child(
            parent,
            self.panel_node(
                rail,
                "editor/workbench/reference/activity_rail",
                fixed_width_box(self.metrics.activity_rail_width),
                UiContainerKind::VerticalBox(UiLinearBoxConfig { gap: 8.0 }),
                "#0f1419",
                Some((self.palette.divider, 1.0)),
            ),
        )?;
        for (index, label) in [
            "Run", "Scene", "Graph", "Image", "Audio", "Code", "Settings",
        ]
        .into_iter()
        .enumerate()
        {
            let id = self.alloc_id();
            self.insert_child(
                rail,
                self.button_node(
                    id,
                    &format!("editor/workbench/reference/activity_rail/{}", index),
                    label,
                    fixed_size_box(self.metrics.activity_rail_width, 48.0),
                    UiWidgetBehavior::Button,
                    index == 0,
                ),
            )?;
        }
        let fill_id = self.alloc_id();
        self.insert_child(
            rail,
            spacer_node(
                fill_id,
                "editor/workbench/reference/activity_rail/fill",
                stretch_box(),
            ),
        )?;
        let help_id = self.alloc_id();
        self.insert_child(
            rail,
            self.button_node(
                help_id,
                "editor/workbench/reference/activity_rail/help",
                "Help",
                fixed_size_box(self.metrics.activity_rail_width, 48.0),
                UiWidgetBehavior::Button,
                false,
            ),
        )
    }

    pub(super) fn build_hierarchy_panel(&mut self, parent: UiNodeId) -> Result<(), UiTreeError> {
        let panel = self.ids.hierarchy_panel;
        self.insert_child(
            parent,
            self.panel_node(
                panel,
                "editor/workbench/reference/hierarchy",
                fixed_width_box(self.metrics.hierarchy_panel_width),
                UiContainerKind::VerticalBox(UiLinearBoxConfig { gap: 8.0 }),
                self.palette.panel_background,
                Some((self.palette.divider, 1.0)),
            ),
        )?;
        let header_id = self.alloc_id();
        self.insert_child(
            panel,
            self.label_node(
                header_id,
                "editor/workbench/reference/hierarchy/header",
                "Scene",
                fixed_height_box(self.metrics.panel_header_height),
                15.0,
                self.palette.text_primary,
            ),
        )?;
        let search_id = self.alloc_id();
        self.insert_child(
            panel,
            self.field_node(
                search_id,
                "editor/workbench/reference/hierarchy/search",
                "Search...",
                fixed_height_box(36.0),
                false,
            ),
        )?;
        for (label, depth, selected) in [
            ("Root", 0, false),
            ("Environment", 1, false),
            ("Lighting", 2, false),
            ("Sky", 2, false),
            ("Level", 1, false),
            ("Geometry", 2, false),
            ("Props", 2, true),
            ("PlayerStart", 1, false),
            ("AudioZone", 1, false),
        ] {
            let id = if selected {
                self.ids.tree_props_row
            } else {
                self.alloc_id()
            };
            self.insert_child(
                panel,
                self.tree_row_node(
                    id,
                    &format!(
                        "editor/workbench/reference/hierarchy/{}",
                        label.to_ascii_lowercase()
                    ),
                    label,
                    depth,
                    selected,
                ),
            )?;
        }
        Ok(())
    }

    pub(super) fn build_viewport_panel(&mut self, parent: UiNodeId) -> Result<(), UiTreeError> {
        let viewport = self.ids.viewport_panel;
        self.insert_child(
            parent,
            self.panel_node(
                viewport,
                "editor/workbench/reference/viewport",
                stretch_box(),
                UiContainerKind::VerticalBox(UiLinearBoxConfig { gap: 0.0 }),
                self.palette.viewport_background,
                Some((self.palette.divider, 1.0)),
            ),
        )?;
        let toolbar = self.alloc_id();
        self.insert_child(
            viewport,
            self.panel_node(
                toolbar,
                "editor/workbench/reference/viewport/toolbar",
                fixed_height_box(self.metrics.toolbar_height),
                UiContainerKind::HorizontalBox(UiLinearBoxConfig { gap: 8.0 }),
                "#141a20",
                None,
            ),
        )?;
        for label in ["Perspective", "Lit", "Gizmo", "Snap 10", "Speed 0.25"] {
            let id = self.alloc_id();
            self.insert_child(
                toolbar,
                self.button_node(
                    id,
                    &format!(
                        "editor/workbench/reference/viewport/toolbar/{}",
                        normalized_reference_path_segment(label)
                    ),
                    label,
                    fixed_size_box(104.0, self.metrics.toolbar_height),
                    UiWidgetBehavior::Button,
                    label == "Gizmo",
                ),
            )?;
        }
        let canvas_id = self.alloc_id();
        self.insert_child(
            viewport,
            self.panel_node(
                canvas_id,
                "editor/workbench/reference/viewport/scene_canvas",
                stretch_box(),
                UiContainerKind::GridBox(UiGridBoxConfig {
                    columns: 12,
                    rows: 6,
                    column_gap: 1.0,
                    row_gap: 1.0,
                }),
                "#1c242b",
                None,
            ),
        )
    }

    pub(super) fn build_inspector_panel(&mut self, parent: UiNodeId) -> Result<(), UiTreeError> {
        let inspector = self.ids.inspector_panel;
        self.insert_child(
            parent,
            self.panel_node(
                inspector,
                "editor/workbench/reference/inspector",
                fixed_width_box(self.metrics.inspector_panel_width),
                UiContainerKind::VerticalBox(UiLinearBoxConfig { gap: 10.0 }),
                self.palette.panel_background,
                Some((self.palette.divider, 1.0)),
            ),
        )?;
        let header_id = self.alloc_id();
        self.insert_child(
            inspector,
            self.label_node(
                header_id,
                "editor/workbench/reference/inspector/header",
                "Inspector",
                fixed_height_box(self.metrics.panel_header_height),
                15.0,
                self.palette.text_primary,
            ),
        )?;
        let object_id = self.alloc_id();
        self.insert_child(
            inspector,
            self.label_node(
                object_id,
                "editor/workbench/reference/inspector/object",
                "Props",
                fixed_height_box(32.0),
                15.0,
                self.palette.text_primary,
            ),
        )?;
        let tag_id = self.alloc_id();
        self.insert_child(
            inspector,
            self.field_node(
                tag_id,
                "editor/workbench/reference/inspector/tag",
                "Untagged",
                fixed_height_box(self.metrics.control_height),
                false,
            ),
        )?;
        for label in [
            "Transform",
            "Position X 128.4",
            "Rotation Y 90",
            "Scale 1.00",
        ] {
            let id = self.alloc_id();
            self.insert_child(
                inspector,
                self.label_node(
                    id,
                    &format!(
                        "editor/workbench/reference/inspector/{}",
                        normalized_reference_path_segment(label)
                    ),
                    label,
                    fixed_height_box(self.metrics.compact_row_height),
                    13.0,
                    self.palette.text_secondary,
                ),
            )?;
        }
        let add_id = self.alloc_id();
        self.insert_child(
            inspector,
            self.button_node(
                add_id,
                "editor/workbench/reference/inspector/add_component",
                "Add Component",
                fixed_height_box(34.0),
                UiWidgetBehavior::Button,
                false,
            ),
        )
    }

    pub(super) fn build_component_gallery(&mut self) -> Result<(), UiTreeError> {
        let gallery = self.ids.component_gallery;
        self.insert_child(
            self.ids.root,
            self.panel_node(
                gallery,
                "editor/workbench/reference/component_gallery",
                fixed_height_box(self.metrics.component_gallery_height()),
                UiContainerKind::HorizontalBox(UiLinearBoxConfig { gap: 16.0 }),
                "#10171b",
                Some((self.palette.divider, 1.0)),
            ),
        )?;
        self.build_button_samples(gallery)?;
        self.build_input_samples(gallery)?;
        self.build_choice_samples(gallery)?;
        self.build_slider_samples(gallery)?;
        self.build_list_samples(gallery)
    }

    fn build_button_samples(&mut self, parent: UiNodeId) -> Result<(), UiTreeError> {
        let group = self.add_gallery_group(parent, "buttons", "Buttons", 210.0)?;
        self.insert_child(
            group,
            self.button_node(
                self.ids.primary_button,
                "editor/workbench/reference/components/buttons/primary",
                "Primary",
                fixed_height_box(self.metrics.control_height),
                UiWidgetBehavior::Button,
                true,
            ),
        )?;
        for label in ["Secondary", "Tertiary", "Outline", "Dropdown"] {
            let id = self.alloc_id();
            self.insert_child(
                group,
                self.button_node(
                    id,
                    &format!(
                        "editor/workbench/reference/components/buttons/{}",
                        label.to_ascii_lowercase()
                    ),
                    label,
                    fixed_height_box(self.metrics.control_height),
                    UiWidgetBehavior::Button,
                    false,
                ),
            )?;
        }
        Ok(())
    }

    fn build_input_samples(&mut self, parent: UiNodeId) -> Result<(), UiTreeError> {
        let group = self.add_gallery_group(parent, "inputs", "Inputs", 230.0)?;
        for (id, label, focused) in [
            (self.alloc_id(), "Text field", false),
            (self.ids.focused_input, "Focused input", true),
            (self.alloc_id(), "Disabled input", false),
        ] {
            self.insert_child(
                group,
                self.field_node(
                    id,
                    &format!(
                        "editor/workbench/reference/components/inputs/{}",
                        normalized_reference_path_segment(label)
                    ),
                    label,
                    fixed_height_box(self.metrics.control_height),
                    focused,
                ),
            )?;
        }
        Ok(())
    }

    fn build_choice_samples(&mut self, parent: UiNodeId) -> Result<(), UiTreeError> {
        let group = self.add_gallery_group(parent, "choices", "Checkboxes", 220.0)?;
        self.insert_child(
            group,
            self.toggle_node(
                self.ids.checkbox,
                "editor/workbench/reference/components/choices/checkbox",
                "Checkbox",
                true,
            ),
        )?;
        let empty_id = self.alloc_id();
        self.insert_child(
            group,
            self.toggle_node(
                empty_id,
                "editor/workbench/reference/components/choices/checkbox_empty",
                "Checkbox",
                false,
            ),
        )?;
        for label in ["Radio option", "Radio option"] {
            let id = self.alloc_id();
            self.insert_child(
                group,
                self.button_node(
                    id,
                    &format!("editor/workbench/reference/components/choices/{}", id.0),
                    label,
                    fixed_height_box(self.metrics.control_height),
                    UiWidgetBehavior::Radio,
                    false,
                ),
            )?;
        }
        Ok(())
    }

    fn build_slider_samples(&mut self, parent: UiNodeId) -> Result<(), UiTreeError> {
        let group = self.add_gallery_group(parent, "sliders", "Sliders", 270.0)?;
        let value_id = self.alloc_id();
        self.insert_child(
            group,
            self.label_node(
                value_id,
                "editor/workbench/reference/components/sliders/value",
                "Value        0.75",
                fixed_height_box(self.metrics.control_height),
                13.0,
                self.palette.text_secondary,
            ),
        )?;
        let track_id = self.alloc_id();
        self.insert_child(
            group,
            self.panel_node(
                track_id,
                "editor/workbench/reference/components/sliders/track",
                fixed_height_box(8.0),
                UiContainerKind::HorizontalBox(UiLinearBoxConfig { gap: 0.0 }),
                self.palette.accent_soft,
                None,
            ),
        )?;
        self.insert_child(
            group,
            self.button_node(
                self.ids.slider_thumb,
                "editor/workbench/reference/components/sliders/thumb",
                "",
                fixed_size_box(22.0, 22.0),
                UiWidgetBehavior::Range,
                true,
            ),
        )?;
        Ok(())
    }

    fn build_list_samples(&mut self, parent: UiNodeId) -> Result<(), UiTreeError> {
        let group = self.add_gallery_group(parent, "lists", "List", 300.0)?;
        for (label, selected) in [
            ("List item", false),
            ("Selected item", true),
            ("Disabled item", false),
        ] {
            let id = if selected {
                self.ids.selected_list_item
            } else {
                self.alloc_id()
            };
            self.insert_child(
                group,
                self.button_node(
                    id,
                    &format!(
                        "editor/workbench/reference/components/list/{}",
                        normalized_reference_path_segment(label)
                    ),
                    label,
                    fixed_height_box(self.metrics.control_height),
                    UiWidgetBehavior::Button,
                    selected,
                ),
            )?;
        }
        Ok(())
    }

    fn add_gallery_group(
        &mut self,
        parent: UiNodeId,
        key: &str,
        title: &str,
        width: f32,
    ) -> Result<UiNodeId, UiTreeError> {
        let group = self.alloc_id();
        self.insert_child(
            parent,
            self.panel_node(
                group,
                &format!("editor/workbench/reference/components/{}", key),
                fixed_width_box(width),
                UiContainerKind::VerticalBox(UiLinearBoxConfig { gap: 8.0 }),
                "#10171b",
                Some((self.palette.divider, 1.0)),
            ),
        )?;
        let title_id = self.alloc_id();
        self.insert_child(
            group,
            self.label_node(
                title_id,
                &format!("editor/workbench/reference/components/{}/title", key),
                title,
                fixed_height_box(28.0),
                13.0,
                self.palette.text_secondary,
            ),
        )?;
        Ok(group)
    }
}
