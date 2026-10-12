#[path = "asset_importer.rs"]
mod asset_importer;
#[path = "auxiliary_source.rs"]
mod auxiliary_source;
#[path = "generate_normals.rs"]
mod generate_normals;
#[path = "gltf_animation_subassets.rs"]
mod gltf_animation_subassets;
#[path = "gltf_decode.rs"]
mod gltf_decode;
#[path = "gltf_labeled_subassets.rs"]
mod gltf_labeled_subassets;
#[path = "gltf_meshopt.rs"]
mod gltf_meshopt;
#[path = "import_animation_asset.rs"]
mod import_animation_asset;
#[path = "import_authoring_asset.rs"]
mod import_authoring_asset;
#[path = "import_cube_lut.rs"]
mod import_cube_lut;
#[path = "import_data_asset.rs"]
mod import_data_asset;
#[cfg(feature = "text")]
#[path = "import_font_asset/mod.rs"]
mod import_font_asset;
#[path = "import_from_source.rs"]
mod import_from_source;
#[path = "import_gltf.rs"]
mod import_gltf;
#[path = "import_material.rs"]
mod import_material;
#[path = "import_mesh.rs"]
mod import_mesh;
#[path = "import_model.rs"]
mod import_model;
#[path = "import_obj.rs"]
mod import_obj;
#[path = "import_physics_material.rs"]
mod import_physics_material;
#[path = "import_scene.rs"]
mod import_scene;
#[cfg(any(feature = "graphics", feature = "target-server"))]
#[path = "import_shader.rs"]
mod import_shader;
#[cfg(any(feature = "graphics", feature = "target-server"))]
#[path = "import_shader_package.rs"]
mod import_shader_package;
#[cfg(test)]
#[path = "tests/import_sound.rs"]
mod import_sound;
#[path = "import_texture.rs"]
mod import_texture;
#[path = "import_ui_icon_asset.rs"]
mod import_ui_icon_asset;
#[path = "import_ui_theme_asset.rs"]
mod import_ui_theme_asset;
#[path = "indexed_mesh_projection.rs"]
mod indexed_mesh_projection;
#[path = "model_mesh_subassets.rs"]
mod model_mesh_subassets;
#[path = "primitive_from_indexed_mesh.rs"]
mod primitive_from_indexed_mesh;
#[cfg(any(feature = "graphics", feature = "target-server"))]
#[path = "validate_wgsl.rs"]
mod validate_wgsl;

pub use asset_importer::AssetImporter;
pub(crate) use auxiliary_source::{open_admitted_file, AuxiliarySourceResolver};
pub use gltf_animation_subassets::add_gltf_animation_and_skin_subassets;
pub(crate) use gltf_decode::snapshot_external_gltf_sources;
pub use gltf_decode::{decode_gltf_source_with_required_extensions, DecodedGltf};
#[cfg(any(feature = "graphics", feature = "target-server"))]
pub(crate) use import_shader_package::snapshot_external_shader_sources;
pub use indexed_mesh_projection::{
    backfill_mesh_sdf_for_model, backfill_virtual_geometry_for_model, cook_mesh_asset_derived_data,
    project_indexed_mesh_primitive, IndexedMeshMissingNormalPolicy, IndexedMeshSource,
};
