use serde::{Deserialize, Serialize};
use toml::Value;

use crate::ui::dispatch::{
    UiNavigationDispatchResult, UiNavigationDispatcher, UiPointerDispatchResult,
    UiPointerDispatcher, UiPointerEvent,
};
use crate::ui::event_ui::{UiNodeId, UiTreeId};
use crate::ui::layout::compute_layout_tree;
use crate::ui::tree::{UiHitTestIndex, UiHitTestResult, UiTemplateNodeMetadata, UiTreeError};
use crate::ui::{layout::UiFrame, layout::UiPoint, layout::UiSize, tree::UiTree};

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct UiFocusState {
    pub focused: Option<UiNodeId>,
    pub captured: Option<UiNodeId>,
    pub hovered: Vec<UiNodeId>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct UiNavigationState {
    pub focus_visible: bool,
    pub navigation_root: Option<UiNodeId>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum UiPointerButton {
    Primary,
    Secondary,
    Middle,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum UiPointerEventKind {
    Down,
    Up,
    Move,
    Scroll,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct UiPointerRoute {
    pub kind: UiPointerEventKind,
    pub button: Option<UiPointerButton>,
    pub point: UiPoint,
    pub scroll_delta: f32,
    pub target: Option<UiNodeId>,
    pub bubbled: Vec<UiNodeId>,
    pub stacked: Vec<UiNodeId>,
    pub entered: Vec<UiNodeId>,
    pub left: Vec<UiNodeId>,
    pub captured: Option<UiNodeId>,
    pub focused: Option<UiNodeId>,
    pub fallback_to_root: bool,
    pub root_targets: Vec<UiNodeId>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum UiNavigationEventKind {
    Activate,
    Cancel,
    Next,
    Previous,
    Up,
    Down,
    Left,
    Right,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct UiNavigationRoute {
    pub kind: UiNavigationEventKind,
    pub target: Option<UiNodeId>,
    pub bubbled: Vec<UiNodeId>,
    pub fallback_to_root: bool,
    pub root_targets: Vec<UiNodeId>,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum UiRenderCommandKind {
    #[default]
    Group,
    Quad,
    Text,
    Image,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum UiVisualAssetRef {
    Icon(String),
    Image(String),
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct UiResolvedStyle {
    pub background_color: Option<String>,
    pub foreground_color: Option<String>,
    pub border_color: Option<String>,
    pub border_width: f32,
    pub corner_radius: f32,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct UiRenderCommand {
    pub node_id: UiNodeId,
    pub kind: UiRenderCommandKind,
    pub frame: UiFrame,
    pub clip_frame: Option<UiFrame>,
    pub z_index: i32,
    pub style: UiResolvedStyle,
    pub text: Option<String>,
    pub image: Option<UiVisualAssetRef>,
    pub opacity: f32,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct UiRenderList {
    pub commands: Vec<UiRenderCommand>,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct UiRenderExtract {
    pub tree_id: UiTreeId,
    pub list: UiRenderList,
}

impl UiRenderExtract {
    pub fn from_tree(tree: &UiTree) -> Self {
        let commands = tree
            .draw_order()
            .into_iter()
            .filter_map(|node_id| {
                let node = tree.node(node_id)?;
                let visual = UiNodeVisualData::resolve(node.template_metadata.as_ref());
                tree.is_visible_in_tree(node_id)
                    .ok()
                    .filter(|visible| *visible)
                    .map(|_| UiRenderCommand {
                        node_id,
                        kind: resolve_command_kind(
                            &visual.style,
                            visual.text.as_ref(),
                            visual.image.as_ref(),
                        ),
                        frame: node.layout_cache.frame,
                        clip_frame: node.layout_cache.clip_frame,
                        z_index: node.z_index,
                        style: visual.style,
                        text: visual.text,
                        image: visual.image,
                        opacity: visual.opacity,
                    })
            })
            .collect();

        Self {
            tree_id: tree.tree_id.clone(),
            list: UiRenderList { commands },
        }
    }
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct UiSurface {
    pub tree: UiTree,
    pub hit_test: UiHitTestIndex,
    pub focus: UiFocusState,
    pub navigation: UiNavigationState,
    pub render_extract: UiRenderExtract,
}

impl UiSurface {
    pub fn new(tree_id: UiTreeId) -> Self {
        Self {
            tree: UiTree::new(tree_id.clone()),
            hit_test: UiHitTestIndex::default(),
            focus: UiFocusState::default(),
            navigation: UiNavigationState::default(),
            render_extract: UiRenderExtract {
                tree_id,
                list: UiRenderList::default(),
            },
        }
    }

    pub fn rebuild(&mut self) {
        self.hit_test.rebuild(&self.tree);
        self.render_extract = UiRenderExtract::from_tree(&self.tree);
    }

    pub fn compute_layout(&mut self, root_size: UiSize) -> Result<(), UiTreeError> {
        compute_layout_tree(&mut self.tree, root_size)?;
        self.rebuild();
        Ok(())
    }

    pub fn hit_test(&self, point: UiPoint) -> UiHitTestResult {
        self.hit_test.hit_test(&self.tree, point)
    }

    pub fn bubble_route(&self, node_id: UiNodeId) -> Result<Vec<UiNodeId>, UiTreeError> {
        self.tree.bubble_route(node_id)
    }

    pub fn focused_route(&self) -> Vec<UiNodeId> {
        self.focus
            .focused
            .and_then(|node_id| self.tree.bubble_route(node_id).ok())
            .unwrap_or_default()
    }

    pub fn focus_node(&mut self, node_id: UiNodeId) -> Result<(), UiTreeError> {
        let node = self
            .tree
            .node(node_id)
            .ok_or(UiTreeError::MissingNode(node_id))?;
        if !(node.state_flags.visible && node.state_flags.enabled && node.state_flags.focusable) {
            return Err(UiTreeError::MissingNode(node_id));
        }
        self.focus.focused = Some(node_id);
        self.navigation.navigation_root = Some(node_id);
        self.navigation.focus_visible = true;
        Ok(())
    }

    pub fn clear_focus(&mut self) {
        self.focus.focused = None;
        self.navigation.navigation_root = None;
        self.navigation.focus_visible = false;
    }

    pub fn capture_pointer(&mut self, node_id: UiNodeId) -> Result<(), UiTreeError> {
        self.tree
            .node(node_id)
            .ok_or(UiTreeError::MissingNode(node_id))?;
        self.focus.captured = Some(node_id);
        Ok(())
    }

    pub fn release_pointer_capture(&mut self) -> Option<UiNodeId> {
        self.focus.captured.take()
    }

    pub fn route_pointer_event(
        &mut self,
        kind: UiPointerEventKind,
        point: UiPoint,
    ) -> Result<UiPointerRoute, UiTreeError> {
        self.route_pointer_event_with_details(kind, point, None, 0.0)
    }

    pub fn dispatch_pointer_event(
        &mut self,
        dispatcher: &UiPointerDispatcher,
        event: UiPointerEvent,
    ) -> Result<UiPointerDispatchResult, UiTreeError> {
        let route = self.route_pointer_event_with_details(
            event.kind,
            event.point,
            event.button,
            event.scroll_delta,
        )?;
        let mut result = dispatcher.dispatch(&self.tree, route.clone())?;
        if let Some(node_id) = result.captured_by {
            self.focus.captured = Some(node_id);
        } else if matches!(event.kind, UiPointerEventKind::Scroll)
            && result.handled_by.is_none()
            && result.blocked_by.is_none()
        {
            let candidates = if !route.stacked.is_empty() {
                route.stacked.as_slice()
            } else {
                route.root_targets.as_slice()
            };
            if let Some(node_id) = self.tree.first_scrollable_in_candidates(candidates)? {
                let _ = self.tree.scroll_by(node_id, event.scroll_delta)?;
                result.handled_by = Some(node_id);
            }
        }
        Ok(result)
    }

    fn route_pointer_event_with_details(
        &mut self,
        kind: UiPointerEventKind,
        point: UiPoint,
        button: Option<UiPointerButton>,
        scroll_delta: f32,
    ) -> Result<UiPointerRoute, UiTreeError> {
        let hit = self.hit_test(point);
        let previous_hovered = self.focus.hovered.clone();
        let captured = self.focus.captured;
        let target = captured.or(hit.top_hit);
        let bubbled = match target {
            Some(node_id) => self.tree.bubble_route(node_id)?,
            None => Vec::new(),
        };

        self.focus.hovered = hit.stacked.clone();
        if matches!(kind, UiPointerEventKind::Down) {
            if let Some(focus_target) = self.tree.first_focusable_in_route(&bubbled)? {
                self.focus_node(focus_target)?;
            }
        }
        if matches!(kind, UiPointerEventKind::Up) {
            self.focus.captured = None;
        }

        Ok(UiPointerRoute {
            kind,
            button,
            point,
            scroll_delta,
            target,
            bubbled,
            stacked: hit.stacked.clone(),
            entered: diff_nodes(&hit.stacked, &previous_hovered),
            left: diff_nodes(&previous_hovered, &hit.stacked),
            captured,
            focused: self.focus.focused,
            fallback_to_root: target.is_none(),
            root_targets: if target.is_none() {
                self.tree.roots.clone()
            } else {
                Vec::new()
            },
        })
    }

    pub fn route_navigation_event(
        &self,
        kind: UiNavigationEventKind,
    ) -> Result<UiNavigationRoute, UiTreeError> {
        let target = self.focus.focused.or(self.navigation.navigation_root);
        let bubbled = match target {
            Some(node_id) => self.tree.bubble_route(node_id)?,
            None => Vec::new(),
        };
        Ok(UiNavigationRoute {
            kind,
            target,
            bubbled,
            fallback_to_root: target.is_none(),
            root_targets: if target.is_none() {
                self.tree.roots.clone()
            } else {
                Vec::new()
            },
        })
    }

    pub fn dispatch_navigation_event(
        &mut self,
        dispatcher: &UiNavigationDispatcher,
        kind: UiNavigationEventKind,
    ) -> Result<UiNavigationDispatchResult, UiTreeError> {
        let route = self.route_navigation_event(kind)?;
        let mut result = dispatcher.dispatch(&self.tree, route.clone())?;
        if result.focus_changed_to.is_none() {
            if let Some(node_id) = self.tree.next_focusable_target(route.target, route.kind)? {
                result.handled_by = Some(route.target.unwrap_or(node_id));
                result.focus_changed_to = Some(node_id);
            }
        }
        if let Some(node_id) = result.focus_changed_to {
            self.focus_node(node_id)?;
        }
        Ok(result)
    }
}

fn diff_nodes(current: &[UiNodeId], previous: &[UiNodeId]) -> Vec<UiNodeId> {
    current
        .iter()
        .filter(|node_id| !previous.contains(node_id))
        .copied()
        .collect()
}

#[derive(Default)]
struct UiNodeVisualData {
    style: UiResolvedStyle,
    text: Option<String>,
    image: Option<UiVisualAssetRef>,
    opacity: f32,
}

impl UiNodeVisualData {
    fn resolve(metadata: Option<&UiTemplateNodeMetadata>) -> Self {
        Self {
            style: resolve_style(metadata),
            text: resolve_text(metadata),
            image: resolve_image(metadata),
            opacity: resolve_opacity(metadata),
        }
    }
}

fn resolve_command_kind(
    style: &UiResolvedStyle,
    text: Option<&String>,
    image: Option<&UiVisualAssetRef>,
) -> UiRenderCommandKind {
    if style.background_color.is_some()
        || style.border_color.is_some()
        || style.border_width > 0.0
        || style.corner_radius > 0.0
    {
        UiRenderCommandKind::Quad
    } else if text.is_some() {
        UiRenderCommandKind::Text
    } else if image.is_some() {
        UiRenderCommandKind::Image
    } else {
        UiRenderCommandKind::Group
    }
}

fn resolve_style(metadata: Option<&UiTemplateNodeMetadata>) -> UiResolvedStyle {
    UiResolvedStyle {
        background_color: resolve_color_attribute(metadata, "background"),
        foreground_color: resolve_color_attribute(metadata, "foreground"),
        border_color: resolve_color_attribute(metadata, "border"),
        border_width: resolve_table_number(metadata, "border", "width")
            .or_else(|| resolve_number_attribute(metadata, "border_width"))
            .unwrap_or(0.0),
        corner_radius: resolve_table_number(metadata, "border", "radius")
            .or_else(|| resolve_number_attribute(metadata, "radius"))
            .or_else(|| resolve_number_attribute(metadata, "corner_radius"))
            .unwrap_or(0.0),
    }
}

fn resolve_text(metadata: Option<&UiTemplateNodeMetadata>) -> Option<String> {
    resolve_string_attribute(metadata, "text")
        .or_else(|| resolve_string_attribute(metadata, "label"))
        .map(str::to_string)
}

fn resolve_image(metadata: Option<&UiTemplateNodeMetadata>) -> Option<UiVisualAssetRef> {
    resolve_string_attribute(metadata, "icon")
        .map(|icon| UiVisualAssetRef::Icon(icon.to_string()))
        .or_else(|| {
            resolve_string_attribute(metadata, "image")
                .map(|image| UiVisualAssetRef::Image(image.to_string()))
        })
}

fn resolve_opacity(metadata: Option<&UiTemplateNodeMetadata>) -> f32 {
    resolve_number_attribute(metadata, "opacity")
        .unwrap_or(1.0)
        .clamp(0.0, 1.0)
}

fn resolve_string_attribute<'a>(
    metadata: Option<&'a UiTemplateNodeMetadata>,
    key: &str,
) -> Option<&'a str> {
    metadata
        .and_then(|metadata| metadata.attributes.get(key))
        .and_then(Value::as_str)
}

fn resolve_number_attribute(metadata: Option<&UiTemplateNodeMetadata>, key: &str) -> Option<f32> {
    metadata
        .and_then(|metadata| metadata.attributes.get(key))
        .and_then(value_as_f32)
}

fn resolve_table_number(
    metadata: Option<&UiTemplateNodeMetadata>,
    table_key: &str,
    value_key: &str,
) -> Option<f32> {
    metadata
        .and_then(|metadata| metadata.attributes.get(table_key))
        .and_then(Value::as_table)
        .and_then(|table| table.get(value_key))
        .and_then(value_as_f32)
}

fn resolve_color_attribute(metadata: Option<&UiTemplateNodeMetadata>, key: &str) -> Option<String> {
    metadata
        .and_then(|metadata| metadata.attributes.get(key))
        .and_then(resolve_color_value)
}

fn resolve_color_value(value: &Value) -> Option<String> {
    match value {
        Value::String(color) => Some(color.clone()),
        Value::Table(table) => table
            .get("color")
            .and_then(Value::as_str)
            .map(str::to_string),
        _ => None,
    }
}

fn value_as_f32(value: &Value) -> Option<f32> {
    value
        .as_float()
        .or_else(|| value.as_integer().map(|value| value as f64))
        .map(|value| value as f32)
}
