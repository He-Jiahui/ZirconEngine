use std::rc::Rc;

use super::super::data::{FrameRect, TemplateNodeFrameData, TemplatePaneNodeData};
use super::template_popup_bounds;
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
