//! Opt-in fixed scene export for real-browser acceptance, never part of a normal test run.
use std::collections::BTreeMap;
use std::path::PathBuf;
use std::sync::Arc;

use crate::asset::artifact::{
    cook_texture_render_artifact, RenderArtifactTextureCookSettings, WebBundleWriter,
    WEBGPU_ASSET_TARGET,
};
use crate::asset::*;
use crate::core::framework::animation::*;
use crate::core::framework::render::RenderMeshTopology;
use crate::core::math::{Mat4, Transform, Vec2, Vec3};
use crate::core::resource::{
    ResourceHandle, ResourceId, ResourceKind, ResourceRecord, UntypedResourceHandle,
};
use crate::scene::components::{
    AnimationPlayerComponent, AnimationSkeletonComponent, CameraComponent, MeshRenderer,
};
use crate::scene::{DynamicScene, World};

fn uri(label: &str) -> AssetUri {
    AssetUri::parse(label).unwrap()
}
fn id(label: &str) -> ResourceId {
    ResourceId::from_locator(&uri(label))
}
fn reference(label: &str) -> AssetReference {
    AssetReference::from_locator(uri(label))
}
fn channel(values: &[(f32, [f32; 3])]) -> AnimationChannelAsset {
    AnimationChannelAsset {
        interpolation: AnimationInterpolationAsset::Linear,
        keys: values
            .iter()
            .map(|(time, value)| AnimationChannelKeyAsset {
                time_seconds: *time,
                value: AnimationChannelValueAsset::Vec3(*value),
                in_tangent: None,
                out_tangent: None,
            })
            .collect(),
    }
}
fn add(writer: &mut WebBundleWriter, label: &str, asset: ImportedAsset, dependencies: &[&str]) {
    let mut record =
        ResourceRecord::new(id(label), asset_kind_for_imported_asset(&asset), uri(label));
    record.dependency_ids = dependencies.iter().map(|label| id(label)).collect();
    writer.add_asset(&record, &asset).unwrap();
}
fn export_root() -> PathBuf {
    let output = PathBuf::from(
        std::env::var_os("ZIRCON_WEB_FIXTURE_OUTPUT")
            .expect("set ZIRCON_WEB_FIXTURE_OUTPUT to a registered cargo-targets output"),
    );
    let parent = output
        .parent()
        .expect("fixture output needs a registered parent")
        .canonicalize()
        .expect("create the registered parent before running this test");
    let candidate = if output.exists() {
        output.canonicalize().unwrap()
    } else {
        parent.join(
            output
                .file_name()
                .expect("fixture output needs a leaf name"),
        )
    };
    let allowed = ["D:/cargo-targets", "E:/cargo-targets", "F:/cargo-targets"]
        .iter()
        .any(|root| {
            PathBuf::from(root)
                .canonicalize()
                .ok()
                .is_some_and(|canonical| candidate.starts_with(&canonical))
        });
    assert!(allowed, "browser acceptance outputs must physically remain under a drive-root cargo-targets directory");
    std::fs::create_dir_all(&candidate).unwrap();
    let output = candidate;
    output
}

#[test]
#[ignore = "exports a registered fixed browser scene; run explicitly with ZIRCON_WEB_FIXTURE_OUTPUT"]
fn export_webgpu_acceptance_fixture() {
    const MODEL: &str = "res://browser/actor.model";
    const MESH: &str = "res://browser/actor.mesh";
    const MATERIAL: &str = "res://browser/checker.material";
    const TEXTURE: &str = "res://browser/checker.texture";
    const SKELETON: &str = "res://browser/actor.skeleton";
    const CLIP: &str = "res://browser/actor.clip";
    const SHADER: &str = "builtin://shader/pbr.wgsl";
    const UI: &str = "res://browser/hud.view";

    let output = export_root();
    let mut writer = WebBundleWriter::new(output.clone()).unwrap();
    let vertices = [
        (Vec3::new(-0.8, -0.6, 0.0), Vec2::new(0.0, 1.0)),
        (Vec3::new(0.8, -0.6, 0.0), Vec2::new(1.0, 1.0)),
        (Vec3::new(0.0, 0.8, 0.0), Vec2::new(0.5, 0.0)),
    ]
    .into_iter()
    .map(|(position, uv)| {
        let mut vertex = MeshVertex::new(position, Vec3::Z, uv);
        vertex.joint_weights = [1.0, 0.0, 0.0, 0.0];
        vertex
    })
    .collect();
    let primitive = ModelPrimitiveAsset {
        vertices,
        indices: vec![0, 1, 2],
        mesh: None,
        mesh_sdf: None,
        virtual_geometry: None,
    };
    let mut mesh = MeshAsset::from_model_primitive(uri(MESH), &primitive);
    mesh.skin = Some(MeshSkinAsset {
        inverse_bind_matrices: vec![Mat4::IDENTITY.to_cols_array_2d(); 64],
    });
    add(&mut writer, MESH, ImportedAsset::Mesh(mesh), &[]);
    // Two primitives reference one mesh and must share the instance's immutable pose snapshot.
    let primitive = || ModelPrimitiveAsset {
        vertices: Vec::new(),
        indices: Vec::new(),
        mesh: Some(reference(MESH)),
        mesh_sdf: None,
        virtual_geometry: None,
    };
    add(
        &mut writer,
        MODEL,
        ImportedAsset::Model(ModelAsset {
            uri: uri(MODEL),
            primitives: vec![primitive(), primitive()],
        }),
        &[MESH],
    );

    let texture = TextureAsset::new_rgba8(
        uri(TEXTURE),
        2,
        2,
        vec![
            240, 80, 30, 255, 30, 120, 240, 255, 30, 120, 240, 255, 240, 80, 30, 255,
        ],
    );
    let cooked = cook_texture_render_artifact(
        UntypedResourceHandle::new(id(TEXTURE), ResourceKind::Texture),
        1,
        texture.clone(),
        RenderArtifactTextureCookSettings::new(
            Arc::from(WEBGPU_ASSET_TARGET),
            0,
            256,
            TextureUploadSupport::uncompressed_only(),
        ),
    )
    .unwrap();
    writer.add_render_artifact(&cooked).unwrap();
    add(&mut writer, TEXTURE, ImportedAsset::Texture(texture), &[]);
    // The engine recognizes this built-in shader identity and assembles its embedded PBR template.
    let shader: ShaderAsset =
        serde_json::from_value(serde_json::json!({"uri":SHADER,"kind":"surface","source":""}))
            .unwrap();
    add(&mut writer, SHADER, ImportedAsset::Shader(shader), &[]);
    let material = MaterialAsset {
        name: Some("Browser checker".into()),
        shader: reference(SHADER),
        parent: None,
        base_color: [1.0; 4],
        base_color_texture: Some(reference(TEXTURE)),
        normal_texture: None,
        metallic: 0.0,
        roughness: 0.7,
        metallic_roughness_texture: None,
        occlusion_texture: None,
        emissive: [0.15; 3],
        emissive_texture: None,
        alpha_mode: AlphaMode::Opaque,
        double_sided: true,
        property_values: BTreeMap::new(),
        texture_slots: BTreeMap::new(),
        options: BTreeMap::new(),
        queue: None,
        validation_diagnostics: Vec::new(),
    };
    add(
        &mut writer,
        MATERIAL,
        ImportedAsset::Material(material),
        &[SHADER, TEXTURE],
    );
    let bones = (0..64)
        .map(|index| AnimationSkeletonBoneAsset {
            name: format!("bone{index}"),
            parent_index: (index != 0).then_some(0),
            local_translation: [0.0; 3],
            local_rotation: [0.0, 0.0, 0.0, 1.0],
            local_scale: [1.0; 3],
        })
        .collect();
    add(
        &mut writer,
        SKELETON,
        ImportedAsset::AnimationSkeleton(AnimationSkeletonAsset {
            name: Some("Browser 64 bones".into()),
            bones,
        }),
        &[],
    );
    add(
        &mut writer,
        CLIP,
        ImportedAsset::AnimationClip(AnimationClipAsset {
            name: Some("Browser root motion".into()),
            skeleton: reference(SKELETON),
            duration_seconds: 2.0,
            tracks: vec![AnimationClipBoneTrackAsset {
                bone_name: "bone0".into(),
                target_id: None,
                translation: channel(&[(0.0, [0.0; 3]), (1.0, [0.25, 0.0, 0.0]), (2.0, [0.0; 3])]),
                rotation: AnimationChannelAsset {
                    interpolation: AnimationInterpolationAsset::Linear,
                    keys: Vec::new(),
                },
                scale: channel(&[(0.0, [1.0; 3])]),
            }],
            event_tracks: Vec::new(),
        }),
        &[SKELETON],
    );
    let view = UiV2ViewAsset::from_toml_str(
        r#"
[asset]
kind = "view"
id = "browser.acceptance.hud"
version = 2
[root]
node = "root"
[nodes.root]
component = "Text"
control_id = "BrowserHud"
props = { text = "Zircon WebGPU 64 bones" }
"#,
    )
    .unwrap();
    add(&mut writer, UI, ImportedAsset::UiV2View(view), &[]);

    let mut world = World::empty();
    let camera = world.spawn((CameraComponent::default(),)).unwrap();
    world
        .update_transform(
            camera,
            Transform {
                translation: Vec3::new(0.0, 0.0, 3.0),
                ..Default::default()
            },
        )
        .unwrap();
    world.set_active_camera(camera);
    world
        .spawn((
            MeshRenderer::from_handles(
                ResourceHandle::new(id(MODEL)),
                ResourceHandle::new(id(MATERIAL)),
            ),
            AnimationSkeletonComponent {
                skeleton: ResourceHandle::new(id(SKELETON)),
            },
            AnimationPlayerComponent {
                clip: ResourceHandle::new(id(CLIP)),
                playback_speed: 1.0,
                time_seconds: 0.0,
                weight: 1.0,
                looping: true,
                playing: true,
            },
        ))
        .unwrap();
    let scene = DynamicScene::from_world(&world).unwrap();
    writer
        .set_startup_scene(
            &scene,
            [MODEL, MATERIAL, SKELETON, CLIP]
                .iter()
                .map(|label| id(label))
                .collect(),
            vec![id(UI)],
        )
        .unwrap();
    let catalog = writer.publish().unwrap();
    assert_eq!(catalog.assets.len(), 8);
    for entry in &catalog.assets {
        let bytes = std::fs::read(output.join(entry.relative_payload_path())).unwrap();
        assert_eq!(entry.payload_bytes, bytes.len() as u64);
        assert_eq!(&entry.content_hash, blake3::hash(&bytes).as_bytes());
        let asset = crate::asset::artifact::read_raw_asset(
            &bytes,
            crate::asset::artifact::RAW_ASSET_MAX_BYTES,
        )
        .unwrap();
        assert_eq!(
            crate::asset::asset_kind_for_imported_asset(&asset),
            entry.record.kind
        );
    }
    println!("WEB_FIXTURE {}", serde_json::to_string(&catalog).unwrap());
}
