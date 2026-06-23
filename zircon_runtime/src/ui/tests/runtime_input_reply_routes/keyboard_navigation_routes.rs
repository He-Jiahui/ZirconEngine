use super::*;
use crate::ui::dispatch::UiInputManager;

mod directional;
mod focus_path;
mod semantic_actions;
mod timers_disabled;

fn horizontal_route_surface() -> UiSurface {
    let mut surface = route_surface();
    surface
        .tree
        .nodes
        .get_mut(&UiNodeId::new(2))
        .unwrap()
        .layout_cache
        .frame = UiFrame::new(10.0, 10.0, 60.0, 30.0);
    surface
        .tree
        .nodes
        .get_mut(&UiNodeId::new(3))
        .unwrap()
        .layout_cache
        .frame = UiFrame::new(90.0, 10.0, 60.0, 30.0);
    surface
}

fn semantic_tabs_route_surface() -> UiSurface {
    let mut surface = route_surface();
    let target = surface.tree.nodes.get_mut(&UiNodeId::new(2)).unwrap();
    target.template_metadata = Some(UiTemplateNodeMetadata {
        component: "Tabs".to_string(),
        control_id: Some("MainTabs".to_string()),
        bindings: vec![binding("Tabs/KeyboardAction", UiEventKind::Change)],
        ..Default::default()
    });
    surface.rebuild();
    surface
}

fn semantic_tree_view_route_surface() -> UiSurface {
    let mut surface = route_surface();
    let target = surface.tree.nodes.get_mut(&UiNodeId::new(2)).unwrap();
    target.template_metadata = Some(UiTemplateNodeMetadata {
        component: "TreeView".to_string(),
        control_id: Some("AssetTree".to_string()),
        bindings: vec![binding("TreeView/KeyboardAction", UiEventKind::Change)],
        ..Default::default()
    });
    surface.rebuild();
    surface
}

fn semantic_menu_list_text_route_surface() -> UiSurface {
    let mut surface = route_surface();
    let target = surface.tree.nodes.get_mut(&UiNodeId::new(2)).unwrap();
    target.template_metadata = Some(UiTemplateNodeMetadata {
        component: "MenuList".to_string(),
        control_id: Some("SceneMenu".to_string()),
        bindings: vec![binding("MenuList/KeyboardText", UiEventKind::Change)],
        ..Default::default()
    });
    surface.rebuild();
    surface
}

fn semantic_menu_list_typeahead_route_surface() -> UiSurface {
    let mut surface = route_surface();
    let target = surface.tree.nodes.get_mut(&UiNodeId::new(2)).unwrap();
    target.template_metadata = Some(UiTemplateNodeMetadata {
        component: "MenuList".to_string(),
        control_id: Some("SceneMenu".to_string()),
        bindings: vec![
            binding("MenuList/KeyboardText", UiEventKind::Change),
            binding("MenuList/TypeaheadExpired", UiEventKind::Change),
        ],
        attributes: toml::from_str("typeahead_timeout_ms = 100").unwrap(),
        ..Default::default()
    });
    surface.rebuild();
    surface
}

fn semantic_menu_list_submenu_hover_route_surface() -> UiSurface {
    let mut surface = route_surface();
    let target = surface.tree.nodes.get_mut(&UiNodeId::new(2)).unwrap();
    target.template_metadata = Some(UiTemplateNodeMetadata {
        component: "MenuList".to_string(),
        control_id: Some("SceneMenu".to_string()),
        bindings: vec![binding("MenuList/ValueChanged", UiEventKind::Change)],
        attributes: toml::from_str("submenu_hover_delay_ms = 100").unwrap(),
        ..Default::default()
    });
    surface.rebuild();
    surface
}

fn semantic_snackbar_toast_route_surface() -> UiSurface {
    let mut surface = route_surface();
    let target = surface.tree.nodes.get_mut(&UiNodeId::new(2)).unwrap();
    target.template_metadata = Some(UiTemplateNodeMetadata {
        component: "Snackbar".to_string(),
        control_id: Some("StatusToast".to_string()),
        bindings: vec![binding("Snackbar/Commit", UiEventKind::Change)],
        attributes: toml::from_str(
            r#"
current_toast_id = "save"
auto_hide_duration_ms = 4000
open = true
"#,
        )
        .unwrap(),
        ..Default::default()
    });
    surface.rebuild();
    surface
}

fn keyboard_navigation_event(shift: bool) -> UiInputEvent {
    let mut metadata = input_metadata();
    metadata.modifiers.shift = shift;
    keyboard_navigation_key_event_with_metadata(metadata, "Tab", 9, Some("\t"))
}

fn keyboard_navigation_key_event(
    logical_key: &str,
    key_code: u32,
    text: Option<&str>,
    shift: bool,
) -> UiInputEvent {
    let mut metadata = input_metadata();
    metadata.modifiers.shift = shift;
    keyboard_navigation_key_event_with_metadata(metadata, logical_key, key_code, text)
}

fn keyboard_navigation_key_event_with_metadata(
    metadata: UiInputEventMetadata,
    logical_key: &str,
    key_code: u32,
    text: Option<&str>,
) -> UiInputEvent {
    UiInputEvent::Keyboard(UiKeyboardInputEvent {
        metadata,
        state: UiKeyboardInputState::Pressed,
        key_code,
        scan_code: None,
        physical_key: logical_key.to_string(),
        logical_key: logical_key.to_string(),
        text: text.map(str::to_string),
    })
}
