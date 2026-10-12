use std::fs;
use std::time::{SystemTime, UNIX_EPOCH};

use crate::core::framework::render::{
    CorePipelineKind, ProjectionMode, RenderAmbientLightSnapshot, RenderMaterialAlphaMode,
    RenderPhase, RenderPhaseMeshSource, RenderSpriteAnchor, RenderSpriteAtlasRegion,
    RenderSpriteImageMode, RenderSpriteRect, RenderSpriteSliceBorder, RenderSpriteSliceScaleMode,
    RenderSpriteSlicer,
};
use crate::core::framework::scene::{ComponentPropertyPath, ScenePropertyValue};
use crate::core::math::{Transform, Vec2, Vec3, Vec4};

use crate::scene::components::{
    CameraComponent, Mesh2dComponent, MeshRenderer, Name, Sprite2dComponent,
};
use crate::scene::{world::World, NodeKind, SystemStage};

use super::authoring_boundary::{
    assert_text_excludes_authoring_tokens, SERIALIZED_AUTHORING_TOKENS,
};
use super::support::{material_handle, model_handle};

#[path = "world_basics/render_extract.rs"]
mod render_extract;
#[path = "world_basics/sprites.rs"]
mod sprites;
#[path = "world_basics/world_state.rs"]
mod world_state;
