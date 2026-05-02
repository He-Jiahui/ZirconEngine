use std::fs;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

use zircon_runtime::asset::{AssetImporter, AssetUri, ImportedAsset};
use zircon_runtime::core::framework::render::{
    RenderMeshSnapshot, RenderVirtualGeometryDebugState,
};
use zircon_runtime::core::framework::scene::Mobility;
use zircon_runtime::core::math::Transform;
use zircon_runtime::core::resource::{MaterialMarker, ModelMarker, ResourceHandle, ResourceId};

use crate::virtual_geometry::build_virtual_geometry_automatic_extract_from_meshes_with_debug;

#[test]
fn virtual_geometry_mesh_based_extract_uses_imported_cooked_model_assets() {
    let root = unique_temp_graphics_root("vg_imported_extract");
    fs::create_dir_all(&root).unwrap();
    let obj_path = root.join("triangle.obj");
    fs::write(
        &obj_path,
        "\
v 0.0 0.0 0.0
v 1.0 0.0 0.0
v 0.0 1.0 0.0
f 1 2 3
",
    )
    .unwrap();
    let model_uri = AssetUri::parse("res://models/imported_triangle.obj").unwrap();
    let imported = AssetImporter::default()
        .import_from_source(&obj_path, &model_uri)
        .unwrap();
    let ImportedAsset::Model(model) = imported else {
        panic!("expected imported OBJ to produce a model asset");
    };
    let model_label = model_uri.to_string();
    let model_id = ResourceId::from_stable_label(&model_label);

    let output = build_virtual_geometry_automatic_extract_from_meshes_with_debug(
        &[RenderMeshSnapshot {
            node_id: 77,
            transform: Transform::default(),
            model: ResourceHandle::<ModelMarker>::new(model_id),
            material: ResourceHandle::<MaterialMarker>::new(ResourceId::from_stable_label(
                "res://materials/imported.material.toml",
            )),
            tint: Default::default(),
            mobility: Mobility::Dynamic,
            render_layer_mask: 1,
        }],
        RenderVirtualGeometryDebugState {
            print_leaf_clusters: true,
            ..Default::default()
        },
        |loaded_model_id| (loaded_model_id == model_id).then(|| model.clone()),
    )
    .expect("imported cooked model should synthesize automatic extract");
    let extract = output.extract();

    assert_eq!(extract.instances.len(), 1);
    assert_eq!(extract.instances[0].entity, 77);
    assert_eq!(extract.instances[0].source_model, Some(model_id));
    assert_eq!(
        extract.instances[0].source_hint.as_deref(),
        Some(model_label.as_str())
    );
    assert!(!extract.clusters.is_empty());
    assert!(!extract.pages.is_empty());
    assert_eq!(output.cpu_reference_instances().len(), 1);
    assert_eq!(output.cpu_reference_instances()[0].entity, 77);

    let _ = fs::remove_dir_all(root);
}

fn unique_temp_graphics_root(label: &str) -> PathBuf {
    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    std::env::temp_dir().join(format!("zircon_graphics_{label}_{unique}"))
}
