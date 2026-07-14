use crate::core::framework::render::MAX_SCREEN_SPACE_TRANSMISSION_STEPS;

pub(crate) const ADVANCED_PBR_OPAQUE_PASS_NAME: &str = "advanced-pbr-opaque";
pub(crate) const ADVANCED_PBR_OPAQUE_EXECUTOR_ID: &str = "mesh.advanced-pbr-opaque";
pub(crate) const TRANSMISSION_SCENE_COPY_PASS_NAME: &str = "transmission.scene_copy";

pub(crate) const TRANSMISSION_SCENE_COPY_EXECUTOR_IDS: [&str; MAX_SCREEN_SPACE_TRANSMISSION_STEPS] = [
    "transmission.scene-copy",
    "transmission.scene-copy.1",
    "transmission.scene-copy.2",
    "transmission.scene-copy.3",
];

pub(crate) const TRANSMISSION_MESH_EXECUTOR_IDS: [&str; MAX_SCREEN_SPACE_TRANSMISSION_STEPS] = [
    "mesh.transmission.0",
    "mesh.transmission.1",
    "mesh.transmission.2",
    "mesh.transmission.3",
];

pub(crate) fn transmission_scene_copy_pass_name(step_index: usize) -> String {
    if step_index == 0 {
        TRANSMISSION_SCENE_COPY_PASS_NAME.to_string()
    } else {
        format!("{TRANSMISSION_SCENE_COPY_PASS_NAME}.{step_index}")
    }
}

pub(crate) fn transmission_mesh_pass_name(step_index: usize) -> String {
    format!("transmission-mesh.{step_index}")
}

pub(crate) fn transmission_scene_copy_step_index(executor_id: &str) -> Option<usize> {
    TRANSMISSION_SCENE_COPY_EXECUTOR_IDS
        .iter()
        .position(|candidate| *candidate == executor_id)
}

pub(crate) fn transmission_mesh_step_index(executor_id: &str) -> Option<usize> {
    TRANSMISSION_MESH_EXECUTOR_IDS
        .iter()
        .position(|candidate| *candidate == executor_id)
}
