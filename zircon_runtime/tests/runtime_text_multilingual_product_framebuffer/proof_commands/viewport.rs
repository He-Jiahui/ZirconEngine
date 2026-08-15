use std::collections::BTreeMap;

use toml::Value;
use zircon_runtime::ui::surface::extract_ui_render_tree_from_arranged;
use zircon_runtime_interface::ui::{
    event_ui::{UiNodeId, UiNodePath, UiTreeId},
    layout::UiFrame,
    surface::{UiArrangedNode, UiArrangedTree, UiRenderCommand},
    tree::{UiInputPolicy, UiTemplateNodeMetadata, UiTree, UiTreeNode, UiVisibility},
};

const LOG_ROW_COUNT: usize = 2_048;
const LOG_LINE_HEIGHT: f32 = 20.0;

pub(in super::super) fn proof_scrolled_plain_text_viewport() -> UiRenderCommand {
    let root_id = UiNodeId::new(130);
    let text_id = UiNodeId::new(131);
    let tree_id = UiTreeId::new("runtime.ui.text.viewport.product");
    let document_frame = UiFrame::new(42.0, 0.0, 996.0, LOG_ROW_COUNT as f32 * LOG_LINE_HEIGHT);
    let clip_frame = UiFrame::new(42.0, 1_961.0, 996.0, 18.0);
    let root_frame = UiFrame::new(0.0, 0.0, 1_080.0, 2_000.0);

    let mut tree = UiTree::new(tree_id.clone());
    tree.insert_root(UiTreeNode::new(root_id, UiNodePath::new("root")));
    tree.insert_child(
        root_id,
        UiTreeNode::new(text_id, UiNodePath::new("root/scrolled-log"))
            .with_template_metadata(log_metadata()),
    )
    .expect("insert viewport proof text node");

    let arranged_tree = UiArrangedTree {
        tree_id,
        roots: vec![root_id],
        nodes: vec![
            arranged_node(root_id, "root", None, vec![text_id], root_frame, root_frame),
            arranged_node(
                text_id,
                "root/scrolled-log",
                Some(root_id),
                Vec::new(),
                document_frame,
                clip_frame,
            ),
        ],
        draw_order: vec![text_id],
        canvas_layers: Vec::new(),
    };
    let extract = extract_ui_render_tree_from_arranged(&tree, &arranged_tree);
    let command = extract
        .list
        .commands
        .into_iter()
        .find(|command| command.node_id == text_id)
        .expect("viewport proof must extract its text command");
    let layout = command
        .text_layout
        .as_ref()
        .expect("viewport proof must resolve text layout");

    assert_eq!(layout.lines.len(), 1);
    assert_eq!(layout.lines[0].text, "log-row-00098");
    assert_eq!(layout.lines[0].frame.y, 1_960.0);
    assert_eq!(layout.measured_height, document_frame.height);
    assert!(!layout.overflow_clipped);
    assert_eq!(command.clip_frame, Some(clip_frame));
    command
}

fn log_metadata() -> UiTemplateNodeMetadata {
    let text = (0..LOG_ROW_COUNT)
        .map(|index| format!("log-row-{index:05}"))
        .collect::<Vec<_>>()
        .join("\n");
    let mut attributes = BTreeMap::new();
    attributes.insert("text".to_string(), Value::String(text));
    attributes.insert(
        "font".to_string(),
        Value::String("res://fonts/default.font.toml".to_string()),
    );
    attributes.insert(
        "foreground_color".to_string(),
        Value::String("#edf6ff".to_string()),
    );
    attributes.insert("font_size".to_string(), Value::Float(16.0));
    attributes.insert(
        "line_height".to_string(),
        Value::Float(f64::from(LOG_LINE_HEIGHT)),
    );
    attributes.insert("wrap".to_string(), Value::String("None".to_string()));
    attributes.insert(
        "text_overflow".to_string(),
        Value::String("Clip".to_string()),
    );

    UiTemplateNodeMetadata {
        component: "Text".to_string(),
        attributes,
        ..UiTemplateNodeMetadata::default()
    }
}

fn arranged_node(
    node_id: UiNodeId,
    node_path: &str,
    parent: Option<UiNodeId>,
    children: Vec<UiNodeId>,
    frame: UiFrame,
    clip_frame: UiFrame,
) -> UiArrangedNode {
    UiArrangedNode {
        node_id,
        node_path: UiNodePath::new(node_path),
        parent,
        children,
        frame,
        clip_frame,
        z_index: 1,
        paint_order: 0,
        visibility: UiVisibility::Visible,
        input_policy: UiInputPolicy::Inherit,
        pointer_events: Default::default(),
        enabled: true,
        clickable: false,
        hoverable: false,
        focusable: false,
        clip_to_bounds: true,
        control_id: None,
        slot: None,
    }
}
