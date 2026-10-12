//! 汇集资产线格式、导入、缓存与项目扫描测试；子模块按资产族组织，共享夹具仅用于构造输入。

#[path = "animation.rs"]
mod animation;
#[path = "artifact_store.rs"]
mod artifact_store;
#[path = "authoring.rs"]
mod authoring;
#[cfg(feature = "text")]
#[path = "font.rs"]
mod font;
#[path = "gltf_external_fixtures.rs"]
mod gltf_external_fixtures;
#[path = "gltf_importer.rs"]
mod gltf_importer;
#[path = "gltf_primitive_fixtures.rs"]
mod gltf_primitive_fixtures;
#[path = "gltf_scene_fixtures.rs"]
mod gltf_scene_fixtures;
#[path = "importer.rs"]
mod importer;
#[path = "management.rs"]
mod management;
#[path = "material.rs"]
mod material;
#[path = "mesh.rs"]
mod mesh;
#[path = "model.rs"]
mod model;
#[path = "navigation.rs"]
mod navigation;
#[path = "obj_importer.rs"]
mod obj_importer;
#[path = "physics_material.rs"]
mod physics_material;
#[path = "render_product.rs"]
mod render_product;
#[path = "scene.rs"]
mod scene;
#[path = "shader_readiness.rs"]
mod shader_readiness;
#[path = "sound.rs"]
mod sound;
#[path = "texture_importer.rs"]
mod texture_importer;
#[path = "texture_upload_readiness.rs"]
mod texture_upload_readiness;
#[path = "ui.rs"]
mod ui;
