use std::fs;

use crate::asset::{
    AssetReference, AssetUri, AssetUuid, ImportedAsset, PrefabInstanceAsset,
    PrefabPropertyOverrideAsset, SceneAsset, SceneCameraTargetAsset, SceneEntityAsset,
    SceneMeshInstanceAsset, SceneMeshLodLevelAsset, SceneMeshPrimitiveBindingAsset,
    SceneMobilityAsset, SceneScriptBindingAsset, TransformAsset,
};
use crate::core::framework::animation::AnimationParameterValue;
use crate::core::framework::render::{
    CorePipelineKind, ProjectionMode, RenderCameraClearColor, RenderCameraTarget,
    RenderViewportRect,
};
use crate::core::framework::scene::physics::{PhysicsCombineRule, PhysicsMaterialMetadata};
use crate::core::math::{Transform, UVec2, Vec2, Vec3};
use crate::scene::components::{
    AmbientLight, CameraComponent, ColliderShape, JointKind, RectLight, RigidBodyType,
};

use crate::scene::components::NodeKind;
use crate::scene::world::{SceneProjectError, World};

use super::authoring_boundary::{
    SERIALIZED_AUTHORING_TOKENS, assert_text_excludes_authoring_tokens,
};
use super::support::{
    create_test_project, project_animation_clip_handle, project_animation_graph_handle,
    project_animation_sequence_handle, project_animation_skeleton_handle,
    project_animation_state_machine_handle, project_material_handle, project_mesh_handle,
    project_model_handle, project_physics_material_handle, unique_temp_project_root,
};

mod hierarchy_sources;
mod mesh_bindings;
mod product_fields;

fn asset_reference(uri: &str) -> AssetReference {
    AssetReference::from_locator(AssetUri::parse(uri).unwrap())
}

fn project_io_source() -> &'static str {
    include_str!("../world/project_io.rs")
}

fn project_io_section<'a>(source: &'a str, start: &str, end: &str) -> &'a str {
    let after_start = source
        .split_once(start)
        .unwrap_or_else(|| panic!("project_io source should contain {start}"))
        .1;
    after_start
        .split_once(end)
        .unwrap_or_else(|| {
            panic!("project_io source section starting at {start} should contain {end}")
        })
        .0
}

fn assert_scene_asset_excludes_authoring_tokens(label: &str, scene: &SceneAsset) {
    let serialized = serde_json::to_string(scene).expect("scene asset should serialize");
    assert_text_excludes_authoring_tokens(label, &serialized, SERIALIZED_AUTHORING_TOKENS);
}
