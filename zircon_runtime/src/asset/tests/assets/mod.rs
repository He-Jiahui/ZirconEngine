//! 汇集资产线格式、导入、缓存与项目扫描测试；子模块按资产族组织，共享夹具仅用于构造输入。

mod animation;
mod artifact_store;
mod authoring;
#[cfg(feature = "text")]
mod font;
mod gltf_external_fixtures;
mod gltf_importer;
mod gltf_primitive_fixtures;
mod gltf_scene_fixtures;
mod importer;
mod management;
mod material;
mod mesh;
mod model;
mod navigation;
mod obj_importer;
mod physics_material;
mod render_product;
mod scene;
mod shader_readiness;
mod sound;
mod texture_importer;
mod texture_upload_readiness;
mod ui;
