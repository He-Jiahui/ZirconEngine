use std::collections::BTreeSet;

use accesskit::{
    Action, ActionData, ActionRequest, Node, NodeId, Point, Rect, Role, TextPosition,
    TextSelection, Toggled, Tree, TreeUpdate,
};
use unicode_segmentation::UnicodeSegmentation;
use zircon_runtime_interface::ui::{
    accessibility::{
        UiA11yCheckedState, UiA11yRole, UiA11yTextSelection, UiAccessibilityAction,
        UiAccessibilityActionRequest, UiAccessibilityActionSource, UiAccessibilityNode,
        UiAccessibilityTreeSnapshot,
    },
    event_ui::UiNodeId,
    layout::{UiFrame, UiPoint},
};

const SYNTHETIC_ROOT_NODE_ID: NodeId = NodeId(u64::MAX);

pub(crate) fn snapshot_to_accesskit_tree_update(
    snapshot: &UiAccessibilityTreeSnapshot,
) -> Option<TreeUpdate> {
    let root = accesskit_root_id(snapshot)?;
    let mut nodes = snapshot
        .nodes
        .iter()
        .map(|node| (accesskit_node_id(node.node_id), accesskit_node(node)))
        .collect::<Vec<_>>();

    if snapshot.roots.len() > 1 {
        nodes.push((SYNTHETIC_ROOT_NODE_ID, synthetic_root_node(snapshot)));
    }

    let node_ids = nodes
        .iter()
        .map(|(node_id, _)| *node_id)
        .collect::<BTreeSet<_>>();
    let focus = snapshot
        .focused
        .map(accesskit_node_id)
        .filter(|focused| node_ids.contains(focused))
        .unwrap_or(root);

    Some(TreeUpdate {
        nodes,
        tree: Some(Tree {
            root,
            toolkit_name: Some("ZirconEngine".to_string()),
            toolkit_version: Some(env!("CARGO_PKG_VERSION").to_string()),
        }),
        focus,
    })
}

pub(crate) fn neutral_action_request_from_accesskit(
    request: &ActionRequest,
    snapshot: &UiAccessibilityTreeSnapshot,
) -> Option<UiAccessibilityActionRequest> {
    let action = neutral_action(request.action)?;
    let (value, numeric_value, text_selection, scroll_offset) =
        action_payload(request.data.as_ref(), snapshot);
    if action == UiAccessibilityAction::SetTextSelection && text_selection.is_none() {
        return None;
    }
    Some(UiAccessibilityActionRequest {
        target: UiNodeId::new(request.target.0),
        action,
        source: UiAccessibilityActionSource::AssistiveTechnology,
        value,
        numeric_value,
        text_selection,
        scroll_offset,
    })
}

pub(crate) const fn accesskit_role(role: UiA11yRole) -> Role {
    match role {
        UiA11yRole::Generic => Role::GenericContainer,
        UiA11yRole::Button => Role::Button,
        UiA11yRole::Checkbox => Role::CheckBox,
        UiA11yRole::Radio => Role::RadioButton,
        UiA11yRole::RadioGroup => Role::RadioGroup,
        UiA11yRole::Slider => Role::Slider,
        UiA11yRole::Text => Role::Label,
        UiA11yRole::TextInput => Role::TextInput,
        UiA11yRole::Image => Role::Image,
        UiA11yRole::List => Role::List,
        UiA11yRole::ListItem => Role::ListItem,
        UiA11yRole::Menu => Role::Menu,
        UiA11yRole::MenuItem => Role::MenuItem,
        UiA11yRole::Tab => Role::Tab,
        UiA11yRole::TabList => Role::TabList,
        UiA11yRole::Panel => Role::Pane,
        UiA11yRole::Dialog => Role::Dialog,
        UiA11yRole::Tooltip => Role::Tooltip,
        UiA11yRole::Scrollbar => Role::ScrollBar,
    }
}

fn accesskit_node_id(node_id: UiNodeId) -> NodeId {
    NodeId(node_id.0)
}

fn accesskit_root_id(snapshot: &UiAccessibilityTreeSnapshot) -> Option<NodeId> {
    if snapshot.roots.len() > 1 {
        Some(SYNTHETIC_ROOT_NODE_ID)
    } else {
        snapshot
            .roots
            .first()
            .copied()
            .or_else(|| snapshot.nodes.first().map(|node| node.node_id))
            .map(accesskit_node_id)
    }
}

fn synthetic_root_node(snapshot: &UiAccessibilityTreeSnapshot) -> Node {
    let mut node = Node::new(Role::Window);
    node.set_label(format!("Zircon UI {}", snapshot.tree_id.0));
    node.set_children(
        snapshot
            .roots
            .iter()
            .copied()
            .map(accesskit_node_id)
            .collect::<Vec<_>>(),
    );
    node
}

fn accesskit_node(source: &UiAccessibilityNode) -> Node {
    let mut node = Node::new(accesskit_role(source.role));
    apply_text_properties(source, &mut node);
    apply_state(source, &mut node);
    apply_actions(source, &mut node);
    apply_relations(source, &mut node);
    if let Some(bounds) = source.bounds {
        node.set_bounds(accesskit_rect(bounds));
    }
    if !source.children.is_empty() {
        node.set_children(
            source
                .children
                .iter()
                .copied()
                .map(accesskit_node_id)
                .collect::<Vec<_>>(),
        );
    }
    node
}

fn apply_text_properties(source: &UiAccessibilityNode, node: &mut Node) {
    if let Some(name) = source.name.as_ref() {
        if source.role == UiA11yRole::Text {
            node.set_value(name.clone());
        } else {
            node.set_label(name.clone());
        }
    }
    if let Some(description) = source.description.as_ref() {
        node.set_description(description.clone());
    }
    if let Some(tooltip) = source.tooltip.as_ref() {
        node.set_tooltip(tooltip.clone());
    }
    if let Some(value) = source.state.value.as_ref() {
        node.set_value(value.clone());
        if let Ok(numeric_value) = value.parse::<f64>() {
            if numeric_value.is_finite() {
                node.set_numeric_value(numeric_value);
            }
        }
    }
}

fn apply_state(source: &UiAccessibilityNode, node: &mut Node) {
    if source.state.hidden {
        node.set_hidden();
    }
    if source.state.disabled {
        node.set_disabled();
    }
    if source.state.selected {
        node.set_selected(true);
    }
    if let Some(expanded) = source.state.expanded {
        node.set_expanded(expanded);
    }
    if let Some(checked) = source.state.checked {
        node.set_toggled(match checked {
            UiA11yCheckedState::False => Toggled::False,
            UiA11yCheckedState::True => Toggled::True,
            UiA11yCheckedState::Mixed => Toggled::Mixed,
        });
    }
}

fn apply_actions(source: &UiAccessibilityNode, node: &mut Node) {
    for action in source.actions.iter().copied() {
        for mapped in accesskit_actions(source.role, action) {
            node.add_action(*mapped);
        }
    }
}

fn accesskit_actions(role: UiA11yRole, action: UiAccessibilityAction) -> &'static [Action] {
    match action {
        UiAccessibilityAction::Activate => &[Action::Click],
        UiAccessibilityAction::Focus => &[Action::Focus],
        UiAccessibilityAction::Increment => &[Action::Increment],
        UiAccessibilityAction::Decrement => &[Action::Decrement],
        UiAccessibilityAction::SetValue => &[Action::SetValue],
        UiAccessibilityAction::ReplaceSelectedText => &[Action::ReplaceSelectedText],
        UiAccessibilityAction::SetTextSelection => &[Action::SetTextSelection],
        UiAccessibilityAction::Expand => &[Action::Expand],
        UiAccessibilityAction::Collapse => &[Action::Collapse],
        UiAccessibilityAction::ScrollTo => &[Action::ScrollIntoView, Action::SetScrollOffset],
        UiAccessibilityAction::Dismiss if role == UiA11yRole::Dialog => &[Action::Blur],
        UiAccessibilityAction::Dismiss if role == UiA11yRole::Tooltip => &[Action::HideTooltip],
        UiAccessibilityAction::Dismiss => &[],
    }
}

fn neutral_action(action: Action) -> Option<UiAccessibilityAction> {
    match action {
        Action::Click => Some(UiAccessibilityAction::Activate),
        Action::Focus => Some(UiAccessibilityAction::Focus),
        Action::Increment => Some(UiAccessibilityAction::Increment),
        Action::Decrement => Some(UiAccessibilityAction::Decrement),
        Action::SetValue => Some(UiAccessibilityAction::SetValue),
        Action::ReplaceSelectedText => Some(UiAccessibilityAction::ReplaceSelectedText),
        Action::SetTextSelection => Some(UiAccessibilityAction::SetTextSelection),
        Action::Expand => Some(UiAccessibilityAction::Expand),
        Action::Collapse => Some(UiAccessibilityAction::Collapse),
        Action::ScrollIntoView | Action::SetScrollOffset => Some(UiAccessibilityAction::ScrollTo),
        Action::Blur | Action::HideTooltip => Some(UiAccessibilityAction::Dismiss),
        Action::CustomAction
        | Action::ShowTooltip
        | Action::ScrollDown
        | Action::ScrollLeft
        | Action::ScrollRight
        | Action::ScrollUp
        | Action::ScrollToPoint
        | Action::SetSequentialFocusNavigationStartingPoint
        | Action::ShowContextMenu => None,
    }
}

fn action_payload(
    data: Option<&ActionData>,
    snapshot: &UiAccessibilityTreeSnapshot,
) -> (
    Option<String>,
    Option<f64>,
    Option<UiA11yTextSelection>,
    Option<UiPoint>,
) {
    match data {
        Some(ActionData::Value(value)) => (Some(value.to_string()), None, None, None),
        Some(ActionData::NumericValue(value)) if value.is_finite() => {
            (None, Some(*value), None, None)
        }
        Some(ActionData::SetTextSelection(selection)) => (
            None,
            None,
            neutral_text_selection(*selection, snapshot),
            None,
        ),
        Some(ActionData::SetScrollOffset(point)) => {
            (None, None, None, Some(neutral_scroll_offset(*point)))
        }
        _ => (None, None, None, None),
    }
}

fn neutral_text_selection(
    selection: TextSelection,
    snapshot: &UiAccessibilityTreeSnapshot,
) -> Option<UiA11yTextSelection> {
    // AccessKit positions are character indexes; the neutral DTO stores UTF-8 byte offsets.
    if selection.anchor.node != selection.focus.node {
        return None;
    }
    let anchor = text_position_byte_offset(selection.anchor, snapshot)?;
    let focus = text_position_byte_offset(selection.focus, snapshot)?;
    Some(UiA11yTextSelection {
        caret: focus,
        anchor,
        focus,
    })
}

fn text_position_byte_offset(
    position: TextPosition,
    snapshot: &UiAccessibilityTreeSnapshot,
) -> Option<usize> {
    let node = snapshot.node(UiNodeId::new(position.node.0))?;
    let text = node.state.value.as_deref().or(node.name.as_deref())?;
    Some(character_index_to_byte_offset(
        text,
        position.character_index,
    ))
}

fn character_index_to_byte_offset(text: &str, character_index: usize) -> usize {
    if character_index == 0 {
        return 0;
    }
    text.grapheme_indices(true)
        .map(|(index, grapheme)| index + grapheme.len())
        .nth(character_index - 1)
        .unwrap_or(text.len())
}

fn neutral_scroll_offset(point: Point) -> UiPoint {
    UiPoint {
        x: finite_f64_to_f32(point.x),
        y: finite_f64_to_f32(point.y),
    }
}

fn finite_f64_to_f32(value: f64) -> f32 {
    if value.is_finite() {
        value as f32
    } else {
        0.0
    }
}

fn apply_relations(source: &UiAccessibilityNode, node: &mut Node) {
    if let Some(labelled_by) = source.labelled_by {
        node.set_labelled_by(vec![accesskit_node_id(labelled_by)]);
    }
    if let Some(label_for) = source.label_for {
        node.set_controls(vec![accesskit_node_id(label_for)]);
    }
}

fn accesskit_rect(frame: UiFrame) -> Rect {
    Rect {
        x0: frame.x as f64,
        y0: frame.y as f64,
        x1: (frame.x + frame.width) as f64,
        y1: (frame.y + frame.height) as f64,
    }
}
