use super::super::super::super::data::FrameRect;
use super::super::super::template_slider_geometry::{
    slider_fill_span, slider_label, slider_percent, slider_range_min_percent, slider_tick_count,
    slider_track_rect, slider_value_label, slider_value_rect,
};
use super::super::slider_style;
use super::super::track::push_slider_ticks;
use super::support::{positioned_slider_node, slider_label_color};
use crate::ui::retained_host::primitives::Color;
use zircon_runtime_interface::ui::surface::MAX_UI_SLIDER_TICK_COUNT;

#[test]
fn workbench_slider_uses_declared_label_value_and_track_proportion() {
    let mut node = positioned_slider_node("WorkbenchInputSlider", 0.75, 8.0, 8.0, 184.0, 30.0);
    node.label_text = "Value".into();
    node.value_text = "0.75".into();
    node.label_color = Color::from_rgb_u8(136, 147, 153);
    node.layout_content_offset_x = -10.0;
    node.layout_first_cell_offset_x = 18.0;

    let rect = FrameRect {
        x: 8.0,
        y: 8.0,
        width: 184.0,
        height: 30.0,
    };
    let value_rect = slider_value_rect(&rect);
    let track_rect = slider_track_rect(
        &rect,
        value_rect.as_ref(),
        slider_label(&node).is_some(),
        &node,
    );

    assert_eq!(slider_label(&node).as_deref(), Some("Value"));
    assert_eq!(slider_value_label(&node, 0.75), "0.75");
    assert_eq!(slider_label_color(&node), [136, 147, 153, 255]);
    assert_eq!(track_rect.x, 68.0);
    assert_eq!(track_rect.width, 80.0);
}

#[test]
fn workbench_range_slider_uses_declared_minimum_span() {
    let mut node = positioned_slider_node("WorkbenchInputRangeSlider", 0.8, 8.0, 8.0, 184.0, 46.0);
    node.layout_second_cell_offset_x = 20.0;

    let range_min = slider_range_min_percent(&node).expect("range minimum");
    let (fill_start, fill_end) = slider_fill_span(slider_percent(&node), Some(range_min));

    assert!((range_min - 0.2).abs() < 0.001);
    assert!((fill_start - 0.2).abs() < 0.001);
    assert!((fill_end - 0.8).abs() < 0.001);
}

#[test]
fn workbench_steps_slider_uses_declared_tick_count() {
    let mut node = positioned_slider_node("WorkbenchInputStepsSlider", 0.86, 8.0, 8.0, 184.0, 30.0);
    node.layout_third_cell_offset_x = 5.0;

    assert_eq!(slider_tick_count(&node), Some(5));
    assert_eq!(slider_range_min_percent(&node), None);
}

#[test]
fn workbench_slider_tick_count_clamps_nonfinite_and_extreme_declarations() {
    let mut node = positioned_slider_node("WorkbenchInputSlider", 0.5, 8.0, 8.0, 184.0, 30.0);

    node.layout_third_cell_offset_x = f32::NAN;
    assert_eq!(slider_tick_count(&node), None);
    node.layout_third_cell_offset_x = -100.0;
    assert_eq!(slider_tick_count(&node), None);
    node.layout_third_cell_offset_x = 1.0;
    assert_eq!(slider_tick_count(&node), None);
    node.layout_third_cell_offset_x = 2.0;
    assert_eq!(slider_tick_count(&node), Some(2));
    node.layout_third_cell_offset_x = f32::INFINITY;
    assert_eq!(slider_tick_count(&node), Some(MAX_UI_SLIDER_TICK_COUNT));
    node.layout_third_cell_offset_x = f32::MAX;
    assert_eq!(slider_tick_count(&node), Some(MAX_UI_SLIDER_TICK_COUNT));
}

#[test]
fn workbench_slider_tick_loop_reclamps_to_track_columns() {
    let node = positioned_slider_node("WorkbenchInputSlider", 0.5, 8.0, 8.0, 184.0, 30.0);
    let style = slider_style(&node);
    let clip = FrameRect {
        x: 0.0,
        y: 0.0,
        width: 640.0,
        height: 64.0,
    };
    let mut commands = Vec::new();

    let narrow_track = FrameRect {
        x: 8.0,
        y: 16.0,
        width: 24.0,
        height: 4.0,
    };
    push_slider_ticks(&mut commands, &narrow_track, &clip, 0, 10_000, &style, 1.0);
    assert_eq!(commands.len(), 24);

    commands.clear();
    let wide_track = FrameRect {
        width: 512.0,
        ..narrow_track
    };
    push_slider_ticks(&mut commands, &wide_track, &clip, 0, 10_000, &style, 1.0);
    assert_eq!(commands.len(), MAX_UI_SLIDER_TICK_COUNT);
}
