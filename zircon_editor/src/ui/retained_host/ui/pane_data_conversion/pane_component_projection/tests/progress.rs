use super::*;

#[test]
fn runtime_progress_projection_maps_track_and_fill_into_native_paint_channels() {
    let progress = host_template_node(projected_node(
        "Progress",
        [
            ("value", Value::Float(64.0)),
            ("value_percent", Value::Float(0.64)),
            ("track_color", Value::String("#151a1d".to_owned())),
            ("fill_color", Value::String("#28b8c5".to_owned())),
        ],
    ))
    .expect("Progress should project its visual channels into the host contract");

    assert_eq!(progress.component_role.as_str(), "progress");
    assert!((progress.value_percent - 0.64).abs() <= f32::EPSILON);
    assert_eq!(
        style_color_u8(progress.button_style.element.background_color.as_ref()),
        Some([21, 26, 29, 255])
    );
    assert_eq!(
        style_color_u8(progress.button_style.element.foreground_color.as_ref()),
        Some([40, 184, 197, 255])
    );
}

#[test]
fn runtime_progress_projection_selects_warning_and_disabled_palette_channels() {
    let warning = host_template_node(projected_node(
        "Progress",
        [
            ("fill_color", Value::String("#28b8c5".to_owned())),
            ("warning_color", Value::String("#d99b2b".to_owned())),
            ("validation_level", Value::String("warning".to_owned())),
        ],
    ))
    .expect("Progress warning state should project its semantic fill color");
    let disabled = host_template_node(projected_node(
        "Progress",
        [
            ("disabled", Value::Boolean(true)),
            ("track_color", Value::String("#151a1d".to_owned())),
            ("disabled_track_color", Value::String("#20262a".to_owned())),
            ("fill_color", Value::String("#28b8c5".to_owned())),
            ("disabled_fill_color", Value::String("#667078".to_owned())),
        ],
    ))
    .expect("Progress disabled state should project its disabled colors");

    assert_eq!(
        style_color_u8(warning.button_style.element.foreground_color.as_ref()),
        Some([217, 155, 43, 255])
    );
    assert_eq!(
        style_color_u8(disabled.button_style.element.background_color.as_ref()),
        Some([32, 38, 42, 255])
    );
    assert_eq!(
        style_color_u8(disabled.button_style.element.foreground_color.as_ref()),
        Some([102, 112, 120, 255])
    );
}

#[test]
fn runtime_progress_projection_selects_the_error_palette_channel() {
    let error = host_template_node(projected_node(
        "Progress",
        [
            ("fill_color", Value::String("#28b8c5".to_owned())),
            ("error_color", Value::String("#d74b4b".to_owned())),
            ("validation_level", Value::String("error".to_owned())),
        ],
    ))
    .expect("Progress error state should project its semantic fill color");

    assert_eq!(
        style_color_u8(error.button_style.element.foreground_color.as_ref()),
        Some([215, 75, 75, 255])
    );
}
