use zircon_runtime_interface::ui::event_ui::UiNodeId;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct EditorWorkbenchReferenceIds {
    pub root: UiNodeId,
    pub top_bar: UiNodeId,
    pub upper_shell: UiNodeId,
    pub activity_rail: UiNodeId,
    pub hierarchy_panel: UiNodeId,
    pub viewport_panel: UiNodeId,
    pub inspector_panel: UiNodeId,
    pub component_gallery: UiNodeId,
    pub status_bar: UiNodeId,
    pub play_button: UiNodeId,
    pub select_tool_button: UiNodeId,
    pub tree_props_row: UiNodeId,
    pub primary_button: UiNodeId,
    pub focused_input: UiNodeId,
    pub checkbox: UiNodeId,
    pub slider_thumb: UiNodeId,
    pub selected_list_item: UiNodeId,
}

impl Default for EditorWorkbenchReferenceIds {
    fn default() -> Self {
        // Stable ids make screenshot comparison, hit routing, and diagnostics diffable.
        Self {
            root: UiNodeId::new(1),
            top_bar: UiNodeId::new(2),
            upper_shell: UiNodeId::new(3),
            activity_rail: UiNodeId::new(4),
            hierarchy_panel: UiNodeId::new(5),
            viewport_panel: UiNodeId::new(6),
            inspector_panel: UiNodeId::new(7),
            component_gallery: UiNodeId::new(8),
            status_bar: UiNodeId::new(9),
            play_button: UiNodeId::new(20),
            select_tool_button: UiNodeId::new(21),
            tree_props_row: UiNodeId::new(30),
            primary_button: UiNodeId::new(40),
            focused_input: UiNodeId::new(41),
            checkbox: UiNodeId::new(42),
            slider_thumb: UiNodeId::new(43),
            selected_list_item: UiNodeId::new(44),
        }
    }
}
