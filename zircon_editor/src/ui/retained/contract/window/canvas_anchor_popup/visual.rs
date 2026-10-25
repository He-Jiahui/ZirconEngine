use super::{CanvasAnchorPopupAddress, FrameRect, TemplatePaneNodeData};
use crate::ui::retained_host::primitives::{ModelRc, PhysicalSize};
use zircon_runtime_interface::ui::design_tokens::EditorTypographyTokens;

// UE CanvasSlotCustomization.cpp: 64 initial preview + 2 px content padding on each side.
const PRESET_CELL: f32 = 68.0;
const POPUP_INSET: f32 = 5.0;
const HEADER: f32 = 24.0;
const FOOTER: f32 = 36.0;
const LABELS: [&str; 16] = [
    "Top Left",
    "Top Centre",
    "Top Right",
    "Top Fill",
    "Centre Left",
    "Centre",
    "Centre Right",
    "Centre Fill",
    "Bottom Left",
    "Bottom Centre",
    "Bottom Right",
    "Bottom Fill",
    "Fill Left",
    "Fill Centre",
    "Fill Right",
    "Fill",
];

pub(crate) struct CanvasAnchorPopupVisual {
    pub(crate) bounds: FrameRect,
    pub(crate) nodes: ModelRc<TemplatePaneNodeData>,
    cells: [FrameRect; 16],
}

impl CanvasAnchorPopupVisual {
    pub(crate) fn new(
        address: &CanvasAnchorPopupAddress,
        size: PhysicalSize,
        scale: f32,
        focused: usize,
        hovered: Option<usize>,
        pressed: Option<usize>,
    ) -> Option<Self> {
        if !scale.is_finite() || scale <= 0.0 {
            return None;
        }
        let cell = PRESET_CELL * scale;
        let inset = POPUP_INSET * scale;
        let width = 4.0 * cell + 2.0 * inset;
        let height = 4.0 * cell + 2.0 * inset + (HEADER + FOOTER) * scale;
        if width > size.width as f32 || height > size.height as f32 {
            return None;
        }
        let x = address
            .opener
            .frame
            .x
            .min(size.width as f32 - width)
            .max(0.0);
        let below = address.opener.frame.y + address.opener.frame.height;
        let y = if below + height <= size.height as f32 {
            below
        } else {
            (address.opener.frame.y - height).max(0.0)
        };
        let bounds = FrameRect {
            x,
            y,
            width,
            height,
        };
        let identity = |suffix: &str| TemplatePaneNodeData {
            node_id: format!("{}/canvas-anchor-popup/{suffix}", address.opener.node_id).into(),
            parent_node_id: address.opener.node_id.clone(),
            source_path: address.opener.source_path.clone(),
            source_node_id: address.opener.source_node_id.clone(),
            instance_path: address.opener.instance_path.clone(),
            inspector_property_field_id: "slot.canvas.anchors".into(),
            ..TemplatePaneNodeData::default()
        };
        let mut nodes = Vec::with_capacity(19);
        nodes.push(TemplatePaneNodeData {
            role: "Panel".into(),
            surface_variant: "surface".into(),
            frame: super::super::super::data::TemplateNodeFrameData {
                x,
                y,
                width,
                height,
            },
            ..identity("panel")
        });
        nodes.push(TemplatePaneNodeData {
            role: "Label".into(),
            text: "Anchors".into(),
            text_tone: "default".into(),
            font_size: EditorTypographyTokens::WORKBENCH_BODY_SIZE,
            frame: super::super::super::data::TemplateNodeFrameData {
                x: x + inset,
                y: y + inset,
                width: width - 2.0 * inset,
                height: HEADER * scale,
            },
            ..identity("title")
        });
        let cells = std::array::from_fn(|index| FrameRect {
            x: x + inset + (index % 4) as f32 * cell,
            y: y + inset + HEADER * scale + (index / 4) as f32 * cell,
            width: cell,
            height: cell,
        });
        for (index, frame) in cells.iter().enumerate() {
            nodes.push(TemplatePaneNodeData {
                role: "Button".into(),
                component_role: "button".into(),
                button_variant: "secondary".into(),
                text: LABELS[index].into(),
                text_align: "center".into(),
                font_size: EditorTypographyTokens::WORKBENCH_BODY_SIZE,
                hovered: hovered == Some(index),
                focused: index == focused,
                focus_visible: index == focused,
                focus_visible_known: true,
                pressed: pressed == Some(index),
                frame: super::super::super::data::TemplateNodeFrameData {
                    x: frame.x,
                    y: frame.y,
                    width: frame.width,
                    height: frame.height,
                },
                inspector_property_item_key: index.to_string().into(),
                ..identity(&format!("preset-{index}"))
            });
        }
        nodes.push(TemplatePaneNodeData {
            role: "Label".into(),
            text: "Shift: pivot   Ctrl: offsets   Esc: cancel".into(),
            font_size: EditorTypographyTokens::WORKBENCH_BODY_SIZE,
            text_tone: "muted".into(),
            frame: super::super::super::data::TemplateNodeFrameData {
                x: x + inset,
                y: y + height - FOOTER * scale - inset,
                width: width - 2.0 * inset,
                height: FOOTER * scale,
            },
            ..identity("modifiers")
        });
        Some(Self {
            bounds,
            nodes: crate::ui::layouts::common::model_rc(nodes),
            cells,
        })
    }

    pub(crate) fn cell_at(&self, x: f32, y: f32) -> Option<usize> {
        self.cells.iter().position(|frame| inside(frame, x, y))
    }

    pub(crate) fn contains(&self, x: f32, y: f32) -> bool {
        inside(&self.bounds, x, y)
    }
}

fn inside(frame: &FrameRect, x: f32, y: f32) -> bool {
    x.is_finite()
        && y.is_finite()
        && x >= frame.x
        && y >= frame.y
        && x < frame.x + frame.width
        && y < frame.y + frame.height
}
