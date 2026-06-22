use std::collections::BTreeMap;

mod activation;
mod model;
mod rendering;
mod surface;
mod transform;

pub(super) use self::model::ProjectedWorldSpace;

pub(super) fn projected_world_space(
    component: &str,
    attributes: &BTreeMap<String, toml::Value>,
) -> ProjectedWorldSpace {
    let transform = transform::projected_world_transform(attributes);
    let surface = surface::projected_world_surface(attributes);
    let rendering = rendering::projected_world_rendering(attributes);

    ProjectedWorldSpace {
        enabled: activation::projected_world_space_enabled(component, attributes),
        position_x: transform.position_x,
        position_y: transform.position_y,
        position_z: transform.position_z,
        rotation_x: transform.rotation_x,
        rotation_y: transform.rotation_y,
        rotation_z: transform.rotation_z,
        scale_x: transform.scale_x,
        scale_y: transform.scale_y,
        scale_z: transform.scale_z,
        width: surface.width,
        height: surface.height,
        pixels_per_meter: surface.pixels_per_meter,
        billboard: rendering.billboard,
        depth_test: rendering.depth_test,
        render_order: rendering.render_order,
        camera_target: rendering.camera_target,
    }
}
