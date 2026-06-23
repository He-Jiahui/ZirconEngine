use super::*;

#[test]
fn importer_emits_bevy_style_gltf_labeled_subassets() {
    let root = unique_temp_project_root("gltf_labeled_subassets");
    fs::create_dir_all(&root).unwrap();
    let gltf_path = write_triangle_gltf(&root);
    let importer = importer_with_first_wave_plugin_fixtures();
    let root_uri = AssetUri::parse("res://models/triangle.gltf").unwrap();
    let outcome = importer
        .import_with_settings(&gltf_path, &root_uri, Default::default())
        .unwrap();

    let root_entry = outcome.root_entry().expect("root gltf entry");
    match &root_entry.asset {
        ImportedAsset::Model(model) => {
            assert_eq!(model.primitives.len(), 1);
            assert_eq!(
                model.primitives[0].mesh.as_ref().unwrap().locator,
                label_uri(&root_uri, "Mesh0/Primitive0")
            );
        }
        other => panic!("unexpected root gltf asset: {other:?}"),
    }
    for label in [
        "Scene0",
        "Node0",
        "Mesh0",
        "Mesh0/Primitive0",
        "Material0",
        "Texture0",
        "DefaultMaterial",
        "Animation0",
        "Skin0",
        "Skin0/Skeleton",
        "Skin0/InverseBindMatrices",
    ] {
        assert!(
            root_entry
                .dependencies
                .contains(&label_uri(&root_uri, label)),
            "root dependencies should include {label}"
        );
        assert!(
            outcome
                .entries
                .iter()
                .any(|entry| entry.locator == label_uri(&root_uri, label)),
            "outcome should include {label}"
        );
    }

    match &entry_for_label(&outcome, &root_uri, "Texture0").asset {
        ImportedAsset::Texture(texture) => {
            assert_eq!(texture.width, 1);
            assert_eq!(texture.height, 1);
            assert_eq!(texture.rgba.len(), 4);
        }
        other => panic!("unexpected Texture0 asset: {other:?}"),
    }
    match &entry_for_label(&outcome, &root_uri, "Material0").asset {
        ImportedAsset::Material(material) => {
            assert_eq!(material.name.as_deref(), Some("TriangleMaterial"));
            assert_eq!(material.base_color, [0.2, 0.3, 0.4, 1.0]);
            assert_eq!(
                material.base_color_texture.as_ref().unwrap().locator,
                label_uri(&root_uri, "Texture0")
            );
        }
        other => panic!("unexpected Material0 asset: {other:?}"),
    }
    match &entry_for_label(&outcome, &root_uri, "Mesh0/Primitive0").asset {
        ImportedAsset::Mesh(mesh) => {
            assert_eq!(mesh.vertex_count().unwrap(), 3);
            assert_eq!(
                mesh.skin
                    .as_ref()
                    .expect("skinned gltf mesh primitive should keep inverse bind matrices")
                    .inverse_bind_matrices,
                vec![identity_bind_matrix()]
            );
            assert_eq!(mesh.morph_targets.len(), 1);
            assert_eq!(
                mesh.morph_targets[0]
                    .attributes
                    .get(MESH_ATTRIBUTE_POSITION),
                Some(&MeshAttributeValues::Float32x3(vec![
                    [0.1, 0.0, 0.0],
                    [0.0, 0.1, 0.0],
                    [0.0, 0.0, 0.1],
                ]))
            );
        }
        other => panic!("unexpected Mesh0/Primitive0 asset: {other:?}"),
    }
    match &entry_for_label(&outcome, &root_uri, "Mesh0").asset {
        ImportedAsset::Model(model) => {
            assert_eq!(model.primitives.len(), 1);
            assert_eq!(
                model.primitives[0].mesh.as_ref().unwrap().locator,
                label_uri(&root_uri, "Mesh0/Primitive0")
            );
        }
        other => panic!("unexpected Mesh0 asset: {other:?}"),
    }
    match &entry_for_label(&outcome, &root_uri, "Node0").asset {
        ImportedAsset::Scene(scene) => {
            let entity = scene.entities.first().expect("node entity");
            assert_eq!(entity.name, "TriangleNode");
            let mesh = entity.mesh.as_ref().expect("node mesh");
            assert_eq!(mesh.model.locator, label_uri(&root_uri, "Mesh0"));
            assert_eq!(mesh.material.locator, label_uri(&root_uri, "Material0"));
            assert_eq!(mesh.morph_weights, vec![0.5]);
            assert_eq!(mesh.primitives.len(), 1);
            assert_eq!(
                mesh.primitives[0].mesh.locator,
                label_uri(&root_uri, "Mesh0/Primitive0")
            );
            assert_eq!(
                mesh.primitives[0].material.locator,
                label_uri(&root_uri, "Material0")
            );
        }
        other => panic!("unexpected Node0 asset: {other:?}"),
    }
    match &entry_for_label(&outcome, &root_uri, "Animation0").asset {
        ImportedAsset::AnimationClip(clip) => {
            assert_eq!(
                clip.skeleton.locator,
                label_uri(&root_uri, "Skin0/Skeleton")
            );
            assert_eq!(clip.duration_seconds, 0.0);
            assert_eq!(clip.tracks.len(), 1);
            let track = &clip.tracks[0];
            assert_eq!(track.bone_name, "Node0:TriangleNode");
            assert_eq!(track.target_id.as_deref(), Some("Node0:TriangleNode"));
            assert_eq!(track.translation.keys.len(), 1);
            assert_eq!(
                track.translation.keys[0].value,
                AnimationChannelValueAsset::Vec3([0.0, 0.0, 0.0])
            );
        }
        other => panic!("unexpected Animation0 asset: {other:?}"),
    }
    match &entry_for_label(&outcome, &root_uri, "Skin0").asset {
        ImportedAsset::Data(data) => {
            assert_eq!(data.format, DataAssetFormat::Json);
            assert_eq!(data.canonical_json["kind"], "gltf_skin");
            assert_eq!(data.canonical_json["skin_index"], 0);
            assert_eq!(data.canonical_json["joint_count"], 1);
            assert_eq!(
                data.canonical_json["joints"][0]["node"],
                label_uri(&root_uri, "Node0").to_string()
            );
            assert_eq!(
                data.canonical_json["inverse_bind_matrices"],
                label_uri(&root_uri, "Skin0/InverseBindMatrices").to_string()
            );
            assert_eq!(
                data.canonical_json["skeleton_asset"],
                label_uri(&root_uri, "Skin0/Skeleton").to_string()
            );
            assert_eq!(data.canonical_json["inverse_bind_matrix_count"], 1);
        }
        other => panic!("unexpected Skin0 asset: {other:?}"),
    }
    match &entry_for_label(&outcome, &root_uri, "Skin0/Skeleton").asset {
        ImportedAsset::AnimationSkeleton(skeleton) => {
            assert_eq!(skeleton.name.as_deref(), Some("Skin0"));
            assert_eq!(skeleton.bones.len(), 1);
            assert_eq!(skeleton.bones[0].name, "Node0:TriangleNode");
            assert_eq!(skeleton.bones[0].parent_index, None);
            assert_eq!(skeleton.bones[0].local_translation, [0.0, 0.0, 0.0]);
        }
        other => panic!("unexpected Skin0/Skeleton asset: {other:?}"),
    }
    match &entry_for_label(&outcome, &root_uri, "Skin0/InverseBindMatrices").asset {
        ImportedAsset::Data(data) => {
            assert_eq!(data.format, DataAssetFormat::Json);
            assert_eq!(data.canonical_json["kind"], "gltf_inverse_bind_matrices");
            assert_eq!(data.canonical_json["matrix_count"], 1);
            assert_eq!(
                data.canonical_json["matrices"][0],
                serde_json::json!(identity_bind_matrix())
            );
        }
        other => panic!("unexpected Skin0/InverseBindMatrices asset: {other:?}"),
    }

    let _ = fs::remove_dir_all(root);
}

#[test]
fn importer_emits_synthetic_skeleton_for_node_animation_without_skin() {
    let root = unique_temp_project_root("gltf_node_animation_subassets");
    fs::create_dir_all(&root).unwrap();
    let gltf_path = write_node_animation_gltf(&root);
    let importer = importer_with_first_wave_plugin_fixtures();
    let root_uri = AssetUri::parse("res://models/node_animation.gltf").unwrap();
    let outcome = importer
        .import_with_settings(&gltf_path, &root_uri, Default::default())
        .unwrap();

    let root_entry = outcome.root_entry().expect("root gltf entry");
    for label in ["Animation0", "Animation0/Skeleton"] {
        assert!(
            root_entry
                .dependencies
                .contains(&label_uri(&root_uri, label)),
            "root dependencies should include {label}"
        );
        assert!(
            outcome
                .entries
                .iter()
                .any(|entry| entry.locator == label_uri(&root_uri, label)),
            "outcome should include {label}"
        );
    }

    match &entry_for_label(&outcome, &root_uri, "Animation0").asset {
        ImportedAsset::AnimationClip(clip) => {
            assert_eq!(clip.name.as_deref(), Some("bob"));
            assert_eq!(
                clip.skeleton.locator,
                label_uri(&root_uri, "Animation0/Skeleton")
            );
            assert_eq!(clip.duration_seconds, 1.0);
            assert_eq!(clip.tracks.len(), 1);
            let track = &clip.tracks[0];
            assert_eq!(track.bone_name, "Node1:Body");
            assert_eq!(track.target_id.as_deref(), Some("Node1:Body"));
            assert_eq!(
                track.translation.interpolation,
                AnimationInterpolationAsset::Linear
            );
            assert_eq!(track.translation.keys.len(), 2);
            assert_eq!(
                track.translation.keys[1].value,
                AnimationChannelValueAsset::Vec3([0.0, 0.5, 0.0])
            );
        }
        other => panic!("unexpected Animation0 asset: {other:?}"),
    }
    match &entry_for_label(&outcome, &root_uri, "Animation0/Skeleton").asset {
        ImportedAsset::AnimationSkeleton(skeleton) => {
            assert_eq!(skeleton.name.as_deref(), Some("bob"));
            assert_eq!(skeleton.bones.len(), 2);
            assert_eq!(skeleton.bones[0].name, "Node0:Root");
            assert_eq!(skeleton.bones[0].parent_index, None);
            assert_eq!(skeleton.bones[1].name, "Node1:Body");
            assert_eq!(skeleton.bones[1].parent_index, Some(0));
            assert_eq!(skeleton.bones[1].local_translation, [0.0, 0.25, 0.0]);
        }
        other => panic!("unexpected Animation0/Skeleton asset: {other:?}"),
    }

    let _ = fs::remove_dir_all(root);
}
