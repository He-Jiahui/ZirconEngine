use super::*;

#[test]
fn runtime_component_projection_preserves_world_space_metadata() {
    let world_surface = host_template_node(projected_node(
        "WorldSpaceSurface",
        [
            ("world_position", float_array([1.0, 2.0, 3.0])),
            ("world_rotation", float_array([10.0, 20.0, 30.0])),
            ("world_scale", float_array([2.0, 2.5, 3.0])),
            ("world_size", float_array([4.0, 2.0, 0.0])),
            ("pixels_per_meter", Value::Float(128.0)),
            ("billboard", Value::Boolean(true)),
            ("depth_test", Value::Boolean(true)),
            ("render_order", Value::Integer(7)),
            ("camera_target", Value::String("viewport-main".to_owned())),
        ],
    ))
    .expect("WorldSpaceSurface should project into the host contract");

    assert!(world_surface.world_space_enabled);
    assert_eq!(world_surface.world_position_x, 1.0);
    assert_eq!(world_surface.world_position_y, 2.0);
    assert_eq!(world_surface.world_position_z, 3.0);
    assert_eq!(world_surface.world_rotation_x, 10.0);
    assert_eq!(world_surface.world_rotation_y, 20.0);
    assert_eq!(world_surface.world_rotation_z, 30.0);
    assert_eq!(world_surface.world_scale_x, 2.0);
    assert_eq!(world_surface.world_scale_y, 2.5);
    assert_eq!(world_surface.world_scale_z, 3.0);
    assert_eq!(world_surface.world_width, 4.0);
    assert_eq!(world_surface.world_height, 2.0);
    assert_eq!(world_surface.world_pixels_per_meter, 128.0);
    assert!(world_surface.world_billboard);
    assert!(world_surface.world_depth_test);
    assert_eq!(world_surface.world_render_order, 7);
    assert_eq!(world_surface.world_camera_target.as_str(), "viewport-main");
}

#[test]
fn disabled_component_does_not_project_world_only_fields() {
    let node = host_template_node(projected_node(
        "Button",
        [
            ("world_position", float_array([1.0, 2.0, 3.0])),
            ("world_scale", float_array([2.0, 2.0, 2.0])),
            ("world_size", float_array([4.0, 2.0, 0.0])),
            ("camera_target", Value::String("viewport-main".to_owned())),
        ],
    ))
    .expect("ordinary component should still project into the host contract");

    assert!(!node.world_space_enabled);
    assert_eq!(node.world_position_x, 0.0);
    assert_eq!(node.world_scale_x, 1.0);
    assert_eq!(node.world_width, 0.0);
    assert_eq!(node.world_camera_target.as_str(), "");
}

#[test]
fn world_size_rejects_invalid_components_without_shifting_positions() {
    use crate::ui::retained_host::ui::pane_data_conversion::pane_value_conversion::value_as_float_array;

    for invalid_index in 0..3 {
        let mut values = vec![Value::Integer(4), Value::Float(2.5), Value::Integer(1)];
        values[invalid_index] = Value::String("invalid component".into());
        let value = Value::Array(values);
        assert_eq!(value_as_float_array(&value), None);
        let node = host_template_node(projected_node("WorldSpaceSurface", [("world_size", value)]))
            .expect("invalid dimensions must not remove the world-space node");
        assert!(node.world_space_enabled);
        assert_eq!((node.world_width, node.world_height), (0.0, 0.0));
    }
    let valid = Value::Array(vec![
        Value::Integer(4),
        Value::Float(2.5),
        Value::Integer(1),
    ]);
    assert_eq!(value_as_float_array(&valid), Some(vec![4.0, 2.5, 1.0]));
    let node =
        host_template_node(projected_node("WorldSpaceSurface", [("world_size", valid)])).unwrap();
    assert_eq!((node.world_width, node.world_height), (4.0, 2.5));
    let empty = Value::Array(Vec::new());
    assert_eq!(value_as_float_array(&empty), None);
    let node =
        host_template_node(projected_node("WorldSpaceSurface", [("world_size", empty)])).unwrap();
    assert_eq!((node.world_width, node.world_height), (0.0, 0.0));
}
