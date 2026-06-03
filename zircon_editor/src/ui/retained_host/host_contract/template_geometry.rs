use super::data::{FrameRect, TemplatePaneNodeData};
use super::frame_geometry::{union_frame, visible_frame};
use crate::ui::retained_host::primitives::ModelRc;

pub(super) fn frame_from_template_node(node: &TemplatePaneNodeData) -> FrameRect {
    FrameRect {
        x: node.frame.x,
        y: node.frame.y,
        width: node.frame.width,
        height: node.frame.height,
    }
}

pub(super) fn template_popup_bounds(
    native_window_bounds: &FrameRect,
    nodes: &ModelRc<TemplatePaneNodeData>,
) -> FrameRect {
    if visible_frame(native_window_bounds) {
        return native_window_bounds.clone();
    }
    template_nodes_bounds(nodes).unwrap_or_else(|| FrameRect {
        x: 0.0,
        y: 0.0,
        width: 1.0,
        height: 1.0,
    })
}

pub(super) fn template_nodes_bounds(nodes: &ModelRc<TemplatePaneNodeData>) -> Option<FrameRect> {
    let mut bounds: Option<FrameRect> = None;
    for row in 0..nodes.row_count() {
        let Some(node) = nodes.row_data(row) else {
            continue;
        };
        let frame = frame_from_template_node(&node);
        if !visible_frame(&frame) {
            continue;
        }
        bounds = Some(match bounds {
            Some(current) => union_frame(&current, &frame),
            None => frame,
        });
    }
    bounds
}

#[cfg(test)]
mod tests {
    use std::rc::Rc;

    use super::super::data::TemplateNodeFrameData;
    use super::*;
    use crate::ui::retained_host::primitives::{ModelRc, VecModel};

    #[test]
    fn template_popup_bounds_prefers_visible_native_window_bounds() {
        let nodes = model_rc(vec![node("Dropdown", 24.0, 36.0, 100.0, 28.0)]);
        let native = rect(0.0, 0.0, 320.0, 180.0);

        assert_eq!(template_popup_bounds(&native, &nodes), native);
    }

    #[test]
    fn template_popup_bounds_falls_back_to_visible_template_node_union() {
        let nodes = model_rc(vec![
            node("Dropdown", 24.0, 36.0, 100.0, 28.0),
            node("Menu", 140.0, 20.0, 88.0, 120.0),
            node("Collapsed", 20.0, 20.0, 0.0, 0.0),
        ]);

        assert_eq!(
            template_popup_bounds(&FrameRect::default(), &nodes),
            rect(24.0, 20.0, 204.0, 120.0)
        );
    }

    fn model_rc<T: Clone + 'static>(items: Vec<T>) -> ModelRc<T> {
        Rc::new(VecModel::from(items)).into()
    }

    fn node(id: &str, x: f32, y: f32, width: f32, height: f32) -> TemplatePaneNodeData {
        TemplatePaneNodeData {
            control_id: id.into(),
            frame: TemplateNodeFrameData {
                x,
                y,
                width,
                height,
            },
            ..TemplatePaneNodeData::default()
        }
    }

    fn rect(x: f32, y: f32, width: f32, height: f32) -> FrameRect {
        FrameRect {
            x,
            y,
            width,
            height,
        }
    }
}
