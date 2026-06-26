use std::fs;
use std::time::{SystemTime, UNIX_EPOCH};

use crate::core::framework::render::{
    ProjectionMode, RenderAmbientLightSnapshot, RenderMaterialAlphaMode, RenderPhase,
    RenderPhaseMeshSource, RenderSpriteAnchor, RenderSpriteAtlasRegion, RenderSpriteImageMode,
    RenderSpriteRect, RenderSpriteSliceBorder, RenderSpriteSliceScaleMode, RenderSpriteSlicer,
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

mod render_extract;
mod sprites;
mod world_state;
