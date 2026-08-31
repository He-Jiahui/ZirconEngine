use std::fs;
use std::path::{Path, PathBuf};

use zircon_runtime::asset::assets::{
    AlphaMode, MaterialAsset, MaterialTextureSlotValue, SceneAsset, SceneCameraAsset,
    SceneDirectionalLightAsset, SceneEntityAsset, SceneMeshInstanceAsset, SceneMobilityAsset,
    TransformAsset,
};
use zircon_runtime::asset::project::AssetMetaDocument;
use zircon_runtime::asset::{
    AssetKind, AssetReference, AssetUri, MeshVertex, ModelAsset, ModelPrimitiveAsset,
};
use zircon_runtime::core::framework::render::{ProjectionMode, RenderMaterialTextureTransform};
use zircon_runtime::core::math::{Transform, Vec2, Vec3};
use zircon_runtime_interface::project::{AssetRef, PersistedAssetReference, RelPath};
use zircon_runtime_interface::resource::ResourceScheme;

const SINGLE_PBR_SPHERE_CENTER: [f32; 3] = [0.0, -0.12, 0.0];
const SINGLE_PBR_SPHERE_SCALE: [f32; 3] = [1.35, 1.35, 1.35];
const SINGLE_PBR_SPHERE_ORTHO_SIZE: f32 = 3.4;
const SINGLE_PBR_SPHERE_PERSPECTIVE_CAMERA_Z: f32 = 4.2;
const SINGLE_PBR_SPHERE_ORTHOGRAPHIC_CAMERA_Z: f32 = 7.0;
const STANDARD_INPUTS_NORMAL_SCALE: f32 = 0.55;
const STANDARD_INPUTS_IOR: f32 = 2.0;
const STANDARD_INPUTS_TEXTURE_TRANSFORM: RenderMaterialTextureTransform =
    RenderMaterialTextureTransform {
        scale: [1.35, 0.85],
        offset: [0.1, -0.065],
        rotation: 0.37,
    };

#[derive(Clone, Copy, Debug)]
pub(super) struct SinglePbrSphereCameraView {
    pub eye: [f32; 3],
    pub target: [f32; 3],
    pub projection_mode: ProjectionMode,
    pub ortho_size: f32,
}

impl SinglePbrSphereCameraView {
    pub(super) const fn front(projection_mode: ProjectionMode) -> Self {
        let camera_z = match projection_mode {
            ProjectionMode::Perspective => SINGLE_PBR_SPHERE_PERSPECTIVE_CAMERA_Z,
            ProjectionMode::Orthographic => SINGLE_PBR_SPHERE_ORTHOGRAPHIC_CAMERA_Z,
        };
        Self {
            eye: [0.0, 0.0, camera_z],
            target: SINGLE_PBR_SPHERE_CENTER,
            projection_mode,
            ortho_size: SINGLE_PBR_SPHERE_ORTHO_SIZE,
        }
    }

    pub(super) const fn perspective_eye(eye: [f32; 3]) -> Self {
        Self {
            eye,
            target: SINGLE_PBR_SPHERE_CENTER,
            projection_mode: ProjectionMode::Perspective,
            ortho_size: SINGLE_PBR_SPHERE_ORTHO_SIZE,
        }
    }

    pub(super) fn perspective_orbit_degrees(yaw_degrees: f32, pitch_degrees: f32) -> Self {
        let yaw = yaw_degrees.to_radians();
        let pitch = pitch_degrees.to_radians();
        let radius = SINGLE_PBR_SPHERE_PERSPECTIVE_CAMERA_Z;
        let cos_pitch = pitch.cos();
        let offset = [
            radius * yaw.sin() * cos_pitch,
            radius * pitch.sin(),
            radius * yaw.cos() * cos_pitch,
        ];
        Self::perspective_eye([
            SINGLE_PBR_SPHERE_CENTER[0] + offset[0],
            SINGLE_PBR_SPHERE_CENTER[1] + offset[1],
            SINGLE_PBR_SPHERE_CENTER[2] + offset[2],
        ])
    }
}

pub(super) fn write_uv_sphere_model(path: PathBuf, model_uri: &str, rings: usize, segments: usize) {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).unwrap();
    }

    let rings = rings.max(3);
    let segments = segments.max(6);
    let mut vertices = Vec::new();
    let mut indices = Vec::new();

    for ring in 0..=rings {
        let theta = std::f32::consts::PI * ring as f32 / rings as f32;
        let y = theta.cos();
        let radius = theta.sin();
        for segment in 0..=segments {
            let phi = std::f32::consts::TAU * segment as f32 / segments as f32;
            let x = radius * phi.cos();
            let z = radius * phi.sin();
            vertices.push(
                MeshVertex::new(
                    Vec3::new(x, y, z),
                    Vec3::new(x, y, z),
                    Vec2::new(segment as f32 / segments as f32, ring as f32 / rings as f32),
                )
                .with_tangent([-phi.sin(), 0.0, phi.cos(), 1.0]),
            );
        }
    }
    for ring in 0..rings {
        for segment in 0..segments {
            let a = (ring * (segments + 1) + segment) as u32;
            let b = a + 1;
            let c = a + (segments + 1) as u32;
            let d = c + 1;
            // Keep triangle winding aligned with the radial normals. The mirror validation renders
            // single-sided spheres, so inward winding would expose the back shell and invert the
            // reflected HDRI vertically and front/back.
            indices.extend_from_slice(&[a, b, c, b, d, c]);
        }
    }

    let model = ModelAsset {
        uri: AssetUri::parse(model_uri).unwrap(),
        primitives: vec![ModelPrimitiveAsset {
            vertices,
            indices,
            mesh: None,
            mesh_sdf: None,
            virtual_geometry: None,
        }],
    };
    fs::write(
        path,
        model
            .to_project_toml_string(persist_fixture_reference)
            .unwrap(),
    )
    .unwrap();
}

pub(super) fn write_pbr_matrix_material(path: PathBuf, metallic: f32, smoothness: f32) {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).unwrap();
    }
    let roughness = (1.0 - smoothness).clamp(
        zircon_runtime::core::framework::render::STANDARD_MATERIAL_MIN_ROUGHNESS,
        1.0,
    );
    let mut material = MaterialAsset {
        name: Some(format!(
            "PBR Matrix M{:.3} S{:.3}",
            metallic.clamp(0.0, 1.0),
            smoothness.clamp(0.0, 1.0)
        )),
        shader: super::asset_reference("builtin://shader/pbr.wgsl"),
        parent: None,
        options: Default::default(),
        queue: None,
        base_color: [0.78, 0.74, 0.66, 1.0],
        base_color_texture: None,
        normal_texture: None,
        metallic,
        roughness,
        metallic_roughness_texture: None,
        occlusion_texture: None,
        emissive: [0.0, 0.0, 0.0],
        emissive_texture: None,
        alpha_mode: AlphaMode::Opaque,
        double_sided: false,
        property_values: Default::default(),
        texture_slots: Default::default(),
        validation_diagnostics: Vec::new(),
    };
    material.property_values.insert(
        "lighting_model".to_string(),
        toml::Value::String("pbr".to_string()),
    );
    material
        .property_values
        .insert("receive_shadows".to_string(), toml::Value::Boolean(false));
    fs::write(
        path,
        material
            .to_project_toml_string(persist_fixture_reference)
            .unwrap(),
    )
    .unwrap();
}

pub(super) fn write_single_pbr_material(
    path: PathBuf,
    name: &str,
    base_color: [f32; 4],
    metallic: f32,
    roughness: f32,
    base_color_texture: Option<&str>,
    normal_texture: Option<&str>,
    metallic_roughness_texture: Option<&str>,
) {
    let material = single_pbr_material(
        name,
        base_color,
        metallic,
        roughness,
        base_color_texture,
        normal_texture,
        metallic_roughness_texture,
    );
    write_material(path, &material);
}

pub(super) fn write_single_pbr_material_with_standard_inputs(
    path: PathBuf,
    name: &str,
    base_color: [f32; 4],
    metallic: f32,
    roughness: f32,
    base_color_texture: Option<&str>,
    normal_texture: Option<&str>,
    metallic_roughness_texture: Option<&str>,
) {
    let mut material = single_pbr_material(
        name,
        base_color,
        metallic,
        roughness,
        base_color_texture,
        normal_texture,
        metallic_roughness_texture,
    );
    apply_standard_pbr_input_coverage(&mut material);
    write_material(path, &material);
}

fn single_pbr_material(
    name: &str,
    base_color: [f32; 4],
    metallic: f32,
    roughness: f32,
    base_color_texture: Option<&str>,
    normal_texture: Option<&str>,
    metallic_roughness_texture: Option<&str>,
) -> MaterialAsset {
    let mut material = MaterialAsset {
        name: Some(name.to_string()),
        shader: super::asset_reference("builtin://shader/pbr.wgsl"),
        parent: None,
        options: Default::default(),
        queue: None,
        base_color,
        base_color_texture: base_color_texture.map(super::asset_reference),
        normal_texture: normal_texture.map(super::asset_reference),
        metallic,
        roughness,
        metallic_roughness_texture: metallic_roughness_texture.map(super::asset_reference),
        occlusion_texture: None,
        emissive: [0.0, 0.0, 0.0],
        emissive_texture: None,
        alpha_mode: AlphaMode::Opaque,
        double_sided: false,
        property_values: Default::default(),
        texture_slots: Default::default(),
        validation_diagnostics: Vec::new(),
    };
    material.property_values.insert(
        "lighting_model".to_string(),
        toml::Value::String("pbr".to_string()),
    );
    material
        .property_values
        .insert("receive_shadows".to_string(), toml::Value::Boolean(false));
    material
}

fn apply_standard_pbr_input_coverage(material: &mut MaterialAsset) {
    for (slot, reference) in [
        ("base_color", material.base_color_texture.clone()),
        ("normal", material.normal_texture.clone()),
        (
            "metallic_roughness",
            material.metallic_roughness_texture.clone(),
        ),
    ] {
        if let Some(reference) = reference {
            material.texture_slots.insert(
                slot.to_string(),
                MaterialTextureSlotValue {
                    reference: Some(reference),
                    fallback: None,
                    transform: Some(STANDARD_INPUTS_TEXTURE_TRANSFORM),
                    uv_channel: 0,
                },
            );
        }
    }
    material.property_values.insert(
        "normal_scale".to_string(),
        toml::Value::Float(STANDARD_INPUTS_NORMAL_SCALE as f64),
    );
    material.property_values.insert(
        "ior".to_string(),
        toml::Value::Float(STANDARD_INPUTS_IOR as f64),
    );
}

fn write_material(path: PathBuf, material: &MaterialAsset) {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).unwrap();
    }
    fs::write(
        path,
        material
            .to_project_toml_string(persist_fixture_reference)
            .unwrap(),
    )
    .unwrap();
}

#[cfg(test)]
mod standard_pbr_input_tests {
    use super::*;
    use zircon_runtime::asset::assets::ZMaterialDocument;

    #[test]
    fn standard_pbr_input_fixture_preserves_texture_transforms_normal_scale_and_ior() {
        let mut material = single_pbr_material(
            "Standard PBR input coverage",
            [1.0, 1.0, 1.0, 1.0],
            0.0,
            1.0,
            Some("res://textures/base_color.png"),
            Some("res://textures/normal.png"),
            Some("res://textures/metallic_roughness.png"),
        );
        apply_standard_pbr_input_coverage(&mut material);

        assert_standard_pbr_input_descriptor(&material);
    }

    #[test]
    fn standard_pbr_input_fixture_survives_project_serialization() {
        let mut material = single_pbr_material(
            "Standard PBR input serialization coverage",
            [1.0, 1.0, 1.0, 1.0],
            0.0,
            1.0,
            Some("res://textures/base_color.png"),
            Some("res://textures/normal.png"),
            Some("res://textures/metallic_roughness.png"),
        );
        apply_standard_pbr_input_coverage(&mut material);

        let document = material
            .to_project_toml_string(persist_fixture_reference)
            .expect("serialize Standard-PBR input fixture as a project material");
        let loaded = MaterialAsset::from_zmaterial_document(
            ZMaterialDocument::from_project_toml_str(&document, restore_fixture_reference)
                .expect("deserialize Standard-PBR input fixture project material"),
        );

        assert_standard_pbr_input_descriptor(&loaded);
    }

    fn assert_standard_pbr_input_descriptor(material: &MaterialAsset) {
        let descriptor = material.standard_material_descriptor();
        assert_eq!(
            descriptor.base_color_texture_transform,
            STANDARD_INPUTS_TEXTURE_TRANSFORM
        );
        assert_eq!(
            descriptor.normal_texture_transform,
            STANDARD_INPUTS_TEXTURE_TRANSFORM
        );
        assert_eq!(
            descriptor.metallic_roughness_texture_transform,
            STANDARD_INPUTS_TEXTURE_TRANSFORM
        );
        assert_eq!(descriptor.normal_scale, STANDARD_INPUTS_NORMAL_SCALE);
        assert_eq!(descriptor.metallic, 0.0);
        assert_eq!(descriptor.advanced_features.ior, STANDARD_INPUTS_IOR);
        assert!(descriptor.advanced_features.dielectric_f0() > 0.04);
        assert!(descriptor.advanced_features.requires_forward_path());
    }

    fn restore_fixture_reference(
        reference: &PersistedAssetReference,
    ) -> Result<AssetReference, zircon_runtime::asset::ReferenceResolutionError> {
        if let Some(locator) = reference.builtin_locator() {
            return Ok(AssetReference::from_locator(locator.clone()));
        }
        let project = reference
            .project_ref()
            .expect("Standard-PBR fixture reference must be project-local");
        let relative_path = project
            .path_hint()
            .as_str()
            .strip_prefix("assets/")
            .expect("Standard-PBR fixture project reference must be rooted below assets");
        let locator = AssetUri::new(
            ResourceScheme::Res,
            relative_path.to_owned(),
            project.sub().map(str::to_owned),
        )
        .expect("Standard-PBR fixture project reference locator");
        Ok(AssetReference::new(project.guid(), locator))
    }
}

fn write_pbr_matrix_asset_metadata(asset_root: &Path) {
    write_fixture_asset_metadata(
        asset_root.join("models/pbr_matrix_sphere.model.toml"),
        "res://models/pbr_matrix_sphere.model.toml",
        AssetKind::Model,
    );
    for row in 0..super::PBR_MATRIX_DIMENSION {
        for column in 0..super::PBR_MATRIX_DIMENSION {
            let uri = format!("res://materials/pbr_matrix_r{row}_c{column}.zmaterial");
            write_fixture_asset_metadata(
                asset_root.join(format!("materials/pbr_matrix_r{row}_c{column}.zmaterial")),
                &uri,
                AssetKind::Material,
            );
        }
    }
}

fn write_fixture_asset_metadata(source_path: PathBuf, uri: &str, asset_kind: AssetKind) {
    let reference = super::asset_reference(uri);
    let file_name = source_path
        .file_name()
        .and_then(|name| name.to_str())
        .expect("fixture asset source file name");
    let meta_path = source_path.with_file_name(format!("{file_name}.zmeta"));
    AssetMetaDocument::new(reference.uuid, reference.locator, asset_kind)
        .save(meta_path)
        .expect("write fixture asset metadata");
}

pub(super) fn write_pbr_matrix_scene(path: PathBuf) {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).unwrap();
    }

    let mut entities =
        Vec::with_capacity(super::PBR_MATRIX_DIMENSION * super::PBR_MATRIX_DIMENSION + 2);
    // The serialized scene carries stable locator UUIDs before the first import. Seed matching
    // sidecars for its external assets so the fixture follows the normal project authoring flow.
    let asset_root = path
        .parent()
        .and_then(Path::parent)
        .expect("PBR matrix scene must reside under the fixture asset root");
    write_pbr_matrix_asset_metadata(asset_root);
    entities.push(camera_entity(
        1,
        "Camera",
        [0.0, 0.0, 8.0],
        ProjectionMode::Orthographic,
        super::PBR_MATRIX_ORTHO_SIZE,
    ));
    entities.push(zero_intensity_key_light_entity(2, "Key Light"));

    let mut entity_id = 10_u64;
    for row in 0..super::PBR_MATRIX_DIMENSION {
        for column in 0..super::PBR_MATRIX_DIMENSION {
            entities.push(SceneEntityAsset {
                entity: entity_id,
                name: format!(
                    "PBR M{:.2} S{:.2}",
                    super::pbr_matrix_axis_value(column),
                    super::pbr_matrix_axis_value(row)
                ),
                parent: None,
                transform: TransformAsset {
                    translation: [
                        super::pbr_matrix_world_x(column),
                        super::pbr_matrix_world_y(row),
                        0.0,
                    ],
                    rotation: [0.0, 0.0, 0.0, 1.0],
                    scale: [
                        super::PBR_MATRIX_SPHERE_SCALE,
                        super::PBR_MATRIX_SPHERE_SCALE,
                        super::PBR_MATRIX_SPHERE_SCALE,
                    ],
                },
                active: true,
                render_layer_mask: 0x0000_0001,
                mobility: SceneMobilityAsset::Dynamic,
                camera: None,
                mesh: Some(SceneMeshInstanceAsset {
                    model: super::asset_reference("res://models/pbr_matrix_sphere.model.toml"),
                    mesh: None,
                    material: super::asset_reference(&format!(
                        "res://materials/pbr_matrix_r{row}_c{column}.zmaterial"
                    )),
                    render_queue: 0,
                    material_queue: 0,
                    order_in_layer: 0,
                    depth_bias: 0.0,
                    morph_weights: Vec::new(),
                    primitives: Vec::new(),
                    lods: Vec::new(),
                }),
                ambient_light: None,
                directional_light: None,
                point_light: None,
                rect_light: None,
                spot_light: None,
                post_process_volume: None,
                rigid_body: None,
                collider: None,
                joint: None,
                animation_skeleton: None,
                animation_player: None,
                animation_sequence_player: None,
                animation_graph_player: None,
                animation_state_machine_player: None,
                terrain: None,
                tilemap: None,
                prefab_instance: None,
                script_bindings: Vec::new(),
            });
            entity_id += 1;
        }
    }

    fs::write(
        path,
        SceneAsset { entities }
            .to_project_toml_string(persist_fixture_reference)
            .unwrap(),
    )
    .unwrap();
}

pub(super) fn write_single_pbr_sphere_scene_with_camera_view(
    path: PathBuf,
    camera_view: SinglePbrSphereCameraView,
) {
    write_single_pbr_sphere_scene_with_camera_view_and_material(
        path,
        camera_view,
        "res://materials/single_metal_sphere.zmaterial",
    );
}

pub(super) fn write_single_pbr_sphere_scene_with_camera_view_and_material(
    path: PathBuf,
    camera_view: SinglePbrSphereCameraView,
    material_uri: &str,
) {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).unwrap();
    }

    let entities = vec![
        camera_entity_with_view(1, "Camera", camera_view),
        SceneEntityAsset {
            entity: 2,
            name: "PBR Sphere".to_string(),
            parent: None,
            transform: TransformAsset {
                translation: SINGLE_PBR_SPHERE_CENTER,
                rotation: [0.0, 0.0, 0.0, 1.0],
                scale: SINGLE_PBR_SPHERE_SCALE,
            },
            active: true,
            render_layer_mask: 0x0000_0001,
            mobility: SceneMobilityAsset::Dynamic,
            camera: None,
            mesh: Some(SceneMeshInstanceAsset {
                model: super::asset_reference("res://models/single_pbr_sphere.model.toml"),
                mesh: None,
                material: super::asset_reference(material_uri),
                render_queue: 0,
                material_queue: 0,
                order_in_layer: 0,
                depth_bias: 0.0,
                morph_weights: Vec::new(),
                primitives: Vec::new(),
                lods: Vec::new(),
            }),
            ambient_light: None,
            directional_light: None,
            point_light: None,
            rect_light: None,
            spot_light: None,
            post_process_volume: None,
            rigid_body: None,
            collider: None,
            joint: None,
            animation_skeleton: None,
            animation_player: None,
            animation_sequence_player: None,
            animation_graph_player: None,
            animation_state_machine_player: None,
            terrain: None,
            tilemap: None,
            prefab_instance: None,
            script_bindings: Vec::new(),
        },
        zero_intensity_key_light_entity(3, "Zero Intensity Key Light"),
    ];

    fs::write(
        path,
        SceneAsset { entities }
            .to_project_toml_string(persist_fixture_reference)
            .unwrap(),
    )
    .unwrap();
}

fn persist_fixture_reference(
    reference: &AssetReference,
) -> Result<PersistedAssetReference, zircon_runtime::asset::ReferenceResolutionError> {
    if reference.locator.scheme() == ResourceScheme::Builtin {
        return Ok(PersistedAssetReference::builtin(reference.locator.clone()));
    }
    let path_hint = RelPath::parse(format!("assets/{}", reference.locator.path()))
        .expect("fixture project asset path");
    let reference = AssetRef::try_new(
        reference.uuid,
        path_hint,
        reference.locator.label().map(str::to_owned),
    )
    .expect("fixture project asset reference");
    Ok(PersistedAssetReference::project(reference))
}

fn camera_entity(
    entity: u64,
    name: &str,
    translation: [f32; 3],
    projection_mode: ProjectionMode,
    ortho_size: f32,
) -> SceneEntityAsset {
    camera_entity_with_view(
        entity,
        name,
        SinglePbrSphereCameraView {
            eye: translation,
            target: [translation[0], translation[1], translation[2] - 1.0],
            projection_mode,
            ortho_size,
        },
    )
}

fn camera_entity_with_view(
    entity: u64,
    name: &str,
    camera_view: SinglePbrSphereCameraView,
) -> SceneEntityAsset {
    let eye = Vec3::new(camera_view.eye[0], camera_view.eye[1], camera_view.eye[2]);
    let target = Vec3::new(
        camera_view.target[0],
        camera_view.target[1],
        camera_view.target[2],
    );
    let transform = Transform::looking_at(eye, target, Vec3::Y);
    SceneEntityAsset {
        entity,
        name: name.to_string(),
        parent: None,
        transform: TransformAsset {
            translation: camera_view.eye,
            rotation: transform.rotation.to_array(),
            scale: [1.0, 1.0, 1.0],
        },
        active: true,
        render_layer_mask: 0x0000_0001,
        mobility: SceneMobilityAsset::Dynamic,
        camera: Some(SceneCameraAsset {
            projection_mode: camera_view.projection_mode,
            fov_y_radians: 60.0_f32.to_radians(),
            ortho_size: camera_view.ortho_size,
            z_near: 0.1,
            z_far: 100.0,
            post_process_settings: None,
            ..SceneCameraAsset::default()
        }),
        mesh: None,
        ambient_light: None,
        directional_light: None,
        point_light: None,
        rect_light: None,
        spot_light: None,
        post_process_volume: None,
        rigid_body: None,
        collider: None,
        joint: None,
        animation_skeleton: None,
        animation_player: None,
        animation_sequence_player: None,
        animation_graph_player: None,
        animation_state_machine_player: None,
        terrain: None,
        tilemap: None,
        prefab_instance: None,
        script_bindings: Vec::new(),
    }
}

fn zero_intensity_key_light_entity(entity: u64, name: &str) -> SceneEntityAsset {
    SceneEntityAsset {
        entity,
        name: name.to_string(),
        parent: None,
        transform: TransformAsset {
            translation: [0.0, 0.0, 0.0],
            rotation: [0.0, 0.0, 0.0, 1.0],
            scale: [1.0, 1.0, 1.0],
        },
        active: true,
        render_layer_mask: 0x0000_0001,
        mobility: SceneMobilityAsset::Dynamic,
        camera: None,
        mesh: None,
        ambient_light: None,
        directional_light: Some(SceneDirectionalLightAsset {
            direction: [-0.35, -0.55, -0.76],
            color: [1.0, 0.96, 0.88],
            intensity: 0.0,
            volumetric: false,
        }),
        point_light: None,
        rect_light: None,
        spot_light: None,
        post_process_volume: None,
        rigid_body: None,
        collider: None,
        joint: None,
        animation_skeleton: None,
        animation_player: None,
        animation_sequence_player: None,
        animation_graph_player: None,
        animation_state_machine_player: None,
        terrain: None,
        tilemap: None,
        prefab_instance: None,
        script_bindings: Vec::new(),
    }
}
