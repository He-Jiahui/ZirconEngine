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
