mod nodes;
mod panels;

use zircon_runtime::ui::surface::UiSurface;
use zircon_runtime_interface::ui::{
    event_ui::{UiNodeId, UiTreeId},
    layout::{UiContainerKind, UiLinearBoxConfig},
    tree::{UiTreeError, UiTreeNode},
    widget::UiWidgetBehavior,
};

use super::{
    EditorWorkbenchReferenceIds, EditorWorkbenchReferenceMetrics, EditorWorkbenchReferencePalette,
    EditorWorkbenchReferenceSurface,
};
use nodes::{fixed_height_box, fixed_size_box, spacer_node, stretch_box};

pub(super) struct ReferenceSurfaceBuilder {
    pub(super) surface: UiSurface,
    pub(super) metrics: EditorWorkbenchReferenceMetrics,
    pub(super) palette: EditorWorkbenchReferencePalette,
    pub(super) ids: EditorWorkbenchReferenceIds,
    next_id: u64,
}

impl ReferenceSurfaceBuilder {
    pub(super) fn new(
        metrics: EditorWorkbenchReferenceMetrics,
        palette: EditorWorkbenchReferencePalette,
        ids: EditorWorkbenchReferenceIds,
    ) -> Self {
        Self {
            surface: UiSurface::new(UiTreeId::new("editor.workbench.reference")),
            metrics,
            palette,
            ids,
            next_id: 1000,
        }
    }

    pub(super) fn build(mut self) -> Result<EditorWorkbenchReferenceSurface, UiTreeError> {
        self.build_root()?;
        Ok(EditorWorkbenchReferenceSurface {
            surface: self.surface,
            ids: self.ids,
            metrics: self.metrics,
            palette: self.palette,
        })
    }

    fn build_root(&mut self) -> Result<(), UiTreeError> {
        self.surface.tree.insert_root(self.panel_node(
            self.ids.root,
            "editor/workbench/reference/root",
            stretch_box(),
            UiContainerKind::VerticalBox(UiLinearBoxConfig { gap: 0.0 }),
            self.palette.app_background,
            None,
        ));
        self.build_top_bar()?;
        self.build_upper_shell()?;
        self.build_component_gallery()?;
        self.build_status_bar()
    }

    fn build_top_bar(&mut self) -> Result<(), UiTreeError> {
        let top_bar = self.ids.top_bar;
        self.insert_child(
            self.ids.root,
            self.panel_node(
                top_bar,
                "editor/workbench/reference/top_bar",
                fixed_height_box(self.metrics.top_bar_height),
                UiContainerKind::HorizontalBox(UiLinearBoxConfig { gap: 10.0 }),
                "#10151a",
                Some((self.palette.divider, 1.0)),
            ),
        )?;

        for label in ["Menu", "New", "Open", "Save", "Undo", "Redo"] {
            let id = self.alloc_id();
            self.insert_child(
                top_bar,
                self.button_node(
                    id,
                    &format!(
                        "editor/workbench/reference/top_bar/{}",
                        label.to_ascii_lowercase()
                    ),
                    label,
                    fixed_size_box(58.0, self.metrics.top_bar_height),
                    UiWidgetBehavior::Button,
                    false,
                ),
            )?;
        }
        self.insert_child(
            top_bar,
            self.button_node(
                self.ids.select_tool_button,
                "editor/workbench/reference/top_bar/select",
                "Select",
                fixed_size_box(76.0, self.metrics.top_bar_height),
                UiWidgetBehavior::Button,
                true,
            ),
        )?;
        for label in ["Move", "Rotate", "Scale", "Snap"] {
            let id = self.alloc_id();
            self.insert_child(
                top_bar,
                self.button_node(
                    id,
                    &format!(
                        "editor/workbench/reference/top_bar/{}",
                        label.to_ascii_lowercase()
                    ),
                    label,
                    fixed_size_box(72.0, self.metrics.top_bar_height),
                    UiWidgetBehavior::Button,
                    false,
                ),
            )?;
        }
        let fill_id = self.alloc_id();
        self.insert_child(
            top_bar,
            spacer_node(
                fill_id,
                "editor/workbench/reference/top_bar/fill",
                stretch_box(),
            ),
        )?;
        self.insert_child(
            top_bar,
            self.button_node(
                self.ids.play_button,
                "editor/workbench/reference/top_bar/play",
                "Play",
                fixed_size_box(82.0, self.metrics.top_bar_height),
                UiWidgetBehavior::Button,
                false,
            ),
        )?;
        for label in ["Grid", "Light", "More"] {
            let id = self.alloc_id();
            self.insert_child(
                top_bar,
                self.button_node(
                    id,
                    &format!(
                        "editor/workbench/reference/top_bar/{}",
                        label.to_ascii_lowercase()
                    ),
                    label,
                    fixed_size_box(70.0, self.metrics.top_bar_height),
                    UiWidgetBehavior::Button,
                    false,
                ),
            )?;
        }
        Ok(())
    }

    fn build_upper_shell(&mut self) -> Result<(), UiTreeError> {
        let upper = self.ids.upper_shell;
        self.insert_child(
            self.ids.root,
            self.panel_node(
                upper,
                "editor/workbench/reference/upper_shell",
                fixed_height_box(self.metrics.upper_region_height),
                UiContainerKind::HorizontalBox(UiLinearBoxConfig { gap: 0.0 }),
                self.palette.app_background,
                None,
            ),
        )?;
        self.build_activity_rail(upper)?;
        self.build_hierarchy_panel(upper)?;
        self.build_viewport_panel(upper)?;
        self.build_inspector_panel(upper)
    }

    fn build_status_bar(&mut self) -> Result<(), UiTreeError> {
        let status = self.ids.status_bar;
        self.insert_child(
            self.ids.root,
            self.panel_node(
                status,
                "editor/workbench/reference/status",
                fixed_height_box(self.metrics.status_bar_height),
                UiContainerKind::HorizontalBox(UiLinearBoxConfig { gap: 18.0 }),
                "#12181e",
                Some((self.palette.divider, 1.0)),
            ),
        )?;
        for (label, color) in [
            ("Ready", self.palette.success),
            ("No Errors", self.palette.text_secondary),
            ("2 Warnings", self.palette.warning),
            ("0 Messages", self.palette.text_secondary),
        ] {
            let id = self.alloc_id();
            self.insert_child(
                status,
                self.label_node(
                    id,
                    &format!(
                        "editor/workbench/reference/status/{}",
                        label.replace(' ', "_").to_ascii_lowercase()
                    ),
                    label,
                    fixed_size_box(140.0, self.metrics.status_bar_height),
                    13.0,
                    color,
                ),
            )?;
        }
        let fill_id = self.alloc_id();
        self.insert_child(
            status,
            spacer_node(
                fill_id,
                "editor/workbench/reference/status/fill",
                stretch_box(),
            ),
        )?;
        let zoom_id = self.alloc_id();
        self.insert_child(
            status,
            self.button_node(
                zoom_id,
                "editor/workbench/reference/status/zoom",
                "100%",
                fixed_size_box(110.0, self.metrics.status_bar_height),
                UiWidgetBehavior::Button,
                false,
            ),
        )
    }

    pub(super) fn insert_child(
        &mut self,
        parent: UiNodeId,
        node: UiTreeNode,
    ) -> Result<(), UiTreeError> {
        self.surface.tree.insert_child(parent, node)
    }

    pub(super) fn alloc_id(&mut self) -> UiNodeId {
        let id = self.next_id;
        self.next_id = self.next_id.saturating_add(1);
        UiNodeId::new(id)
    }
}
