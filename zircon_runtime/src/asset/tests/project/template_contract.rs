use crate::asset::{
    AssetReference, AssetUri, AssetUuid, ProjectDocumentError, ReferenceResolutionError,
    SceneAsset, SceneMobilityAsset, ZMaterialDocument,
};
use zircon_runtime_interface::project::{render_project_template, ProjectTemplateId};

#[test]
fn renderable_empty_template_parses_through_runtime_scene_schema_with_project_references() {
    let document = renderable_empty_scene_document();
    let document = std::str::from_utf8(&document).expect("F2 product scene must be UTF-8");
    let persisted: toml::Value =
        toml::from_str(document).expect("F2 product scene must be valid TOML");
    let cube = persisted
        .get("entities")
        .and_then(toml::Value::as_array)
        .and_then(|entities| {
            entities.iter().find(|entity| {
                entity
                    .get("name")
                    .and_then(toml::Value::as_str)
                    .is_some_and(|name| name == "Cube")
            })
        })
        .expect("F2 persisted scene must contain Cube");
    let mesh = cube
        .get("mesh")
        .expect("F2 persisted Cube must contain mesh references");
    assert_template_project_reference(
        mesh.get("model")
            .expect("F2 persisted Cube must contain a model reference"),
        "00000000-0000-0000-0000-000000000002",
        "assets/models/cube.obj",
    );
    assert_template_project_reference(
        mesh.get("material")
            .expect("F2 persisted Cube must contain a material reference"),
        "00000000-0000-0000-0000-000000000003",
        "assets/materials/default.zmaterial",
    );

    let scene = SceneAsset::from_project_toml_str(document, |reference| {
        Ok(resolve_template_project_reference(reference))
    })
    .expect("F2 product scene must parse through the runtime schema");

    assert_eq!(scene.entities.len(), 3);
    assert!(scene.entities.iter().all(|entity| entity.active));

    let camera = scene
        .entities
        .iter()
        .find(|entity| entity.name == "Camera")
        .expect("F2 Camera entity");
    assert_eq!(camera.entity, 1);
    assert_eq!(camera.parent, None);
    assert_eq!(camera.transform.translation, [21.0, 2.0, 14.5]);
    assert_eq!(camera.transform.rotation, [0.0, 0.0, 0.0, 1.0]);
    assert_eq!(camera.transform.scale, [1.0, 1.0, 1.0]);
    let camera_component = camera.camera.as_ref().expect("F2 Camera component");
    assert_eq!(camera_component.fov_y_radians, 1.7453293);
    assert_eq!(camera_component.z_near, 0.1);
    assert_eq!(camera_component.z_far, 200.0);

    let sun = scene
        .entities
        .iter()
        .find(|entity| entity.name == "Sun")
        .expect("F2 Sun entity");
    assert_eq!(sun.entity, 2);
    assert_eq!(sun.parent, None);
    assert_eq!(sun.mobility, SceneMobilityAsset::Static);
    assert_eq!(sun.transform.translation, [0.0, 4.0, 0.0]);
    assert_eq!(sun.transform.rotation, [0.0, 0.0, 0.0, 1.0]);
    assert_eq!(sun.transform.scale, [1.0, 1.0, 1.0]);
    let directional_light = sun.directional_light.as_ref().expect("F2 Sun light");
    assert_eq!(
        directional_light.direction,
        [-0.361772505316908, -0.904431263292269, -0.226107815823067]
    );
    assert_eq!(directional_light.intensity, 3.0);

    let cube = scene
        .entities
        .iter()
        .find(|entity| entity.name == "Cube")
        .expect("F2 Cube entity");
    assert_eq!(cube.entity, 3);
    assert_eq!(cube.parent, None);
    assert_eq!(cube.mobility, SceneMobilityAsset::Static);
    assert_eq!(cube.transform.translation, [0.0, 0.0, 0.0]);
    assert_eq!(cube.transform.rotation, [0.0, 0.0, 0.0, 1.0]);
    assert_eq!(cube.transform.scale, [1.0, 1.0, 1.0]);
    let mesh = cube.mesh.as_ref().expect("F2 Cube mesh binding");
    assert_eq!(
        mesh.model.locator,
        AssetUri::parse("res://models/cube.obj").expect("F2 cube model locator")
    );
    assert_eq!(
        mesh.model.uuid,
        "00000000-0000-0000-0000-000000000002"
            .parse::<AssetUuid>()
            .expect("F2 cube model UUID")
    );
    assert_eq!(
        mesh.material.locator,
        AssetUri::parse("res://materials/default.zmaterial").expect("F2 material locator")
    );
    assert_eq!(
        mesh.material.uuid,
        "00000000-0000-0000-0000-000000000003"
            .parse::<AssetUuid>()
            .expect("F2 material UUID")
    );
}

#[test]
fn renderable_empty_camera_frames_the_baseline_and_authored_cube_positions() {
    const F2_CAPTURE_WIDTH: f32 = 640.0;
    const F2_CAPTURE_HEIGHT: f32 = 360.0;
    const NARROW_DESKTOP_ASPECT_RATIO: f32 = 4.0 / 3.0;
    const CUBE_HALF_EXTENT: f32 = 0.5;
    const REQUIRED_NON_BACKGROUND_PIXELS: f32 = 100.0;

    let document = renderable_empty_scene_document();
    let document = std::str::from_utf8(&document).expect("F2 product scene must be UTF-8");
    let scene = SceneAsset::from_project_toml_str(document, |reference| {
        Ok(resolve_template_project_reference(reference))
    })
    .expect("F2 product scene must parse through the runtime schema");
    let camera = scene
        .entities
        .iter()
        .find(|entity| entity.name == "Camera")
        .expect("F2 Camera entity");
    let camera_component = camera.camera.as_ref().expect("F2 Camera component");

    assert_eq!(camera.transform.rotation, [0.0, 0.0, 0.0, 1.0]);
    let nearest_cube_depth = camera.transform.translation[2] - CUBE_HALF_EXTENT;
    assert!(nearest_cube_depth > camera_component.z_near);
    let half_fov_tangent = (camera_component.fov_y_radians * 0.5).tan();
    assert!(half_fov_tangent.is_finite() && half_fov_tangent > 0.0);

    for cube_x in [0.0, 42.0] {
        let horizontal_offset = cube_x - camera.transform.translation[0];
        let vertical_offset = -camera.transform.translation[1];
        let half_height = nearest_cube_depth * half_fov_tangent;
        let half_width = half_height * NARROW_DESKTOP_ASPECT_RATIO;

        assert!(
            horizontal_offset.abs() + CUBE_HALF_EXTENT <= half_width,
            "Cube X={cube_x} must remain fully visible after F4 authoring"
        );
        assert!(
            vertical_offset.abs() + CUBE_HALF_EXTENT <= half_height,
            "Cube X={cube_x} must remain vertically visible"
        );
    }

    let projected_face_side_pixels =
        (F2_CAPTURE_HEIGHT * 0.5) / (nearest_cube_depth * half_fov_tangent);
    assert!(
        projected_face_side_pixels * projected_face_side_pixels > REQUIRED_NON_BACKGROUND_PIXELS,
        "the unit Cube must retain enough F2 capture coverage at {F2_CAPTURE_WIDTH}x{F2_CAPTURE_HEIGHT}"
    );
}

#[test]
fn renderable_empty_template_preserves_missing_model_reference_as_a_typed_error() {
    assert_template_reference_failure("assets/models/cube.obj");
}

#[test]
fn renderable_empty_template_preserves_missing_material_reference_as_a_typed_error() {
    assert_template_reference_failure("assets/materials/default.zmaterial");
}

#[test]
fn renderable_empty_template_material_targets_the_persisted_project_shader() {
    let document = renderable_empty_template_entry("assets/materials/default.zmaterial");
    let document = std::str::from_utf8(&document).expect("F2 product material must be UTF-8");
    let persisted: toml::Value =
        toml::from_str(document).expect("F2 product material must be valid TOML");
    let shader = persisted
        .get("shader")
        .and_then(toml::Value::as_table)
        .expect("F2 product material shader reference");

    assert_eq!(
        shader.get("kind").and_then(toml::Value::as_str),
        Some("project")
    );
    assert_eq!(
        shader.get("guid").and_then(toml::Value::as_str),
        Some("00000000-0000-0000-0000-000000000001")
    );
    assert_eq!(
        shader.get("path_hint").and_then(toml::Value::as_str),
        Some("assets/shaders/pbr_shader.zmeta")
    );

    let material = ZMaterialDocument::from_project_toml_str(document, |reference| {
        Ok(resolve_template_project_reference(reference))
    })
    .expect("F2 product material must resolve its persisted shader reference");

    assert_eq!(
        material.shader.locator,
        AssetUri::parse("res://shaders/pbr_shader").expect("F2 project shader locator")
    );
    assert_eq!(
        material.shader.uuid,
        "00000000-0000-0000-0000-000000000001"
            .parse::<AssetUuid>()
            .expect("F2 project shader UUID")
    );
}

fn assert_template_reference_failure(missing_path_hint: &str) {
    let document = renderable_empty_scene_document();
    let document = std::str::from_utf8(&document).expect("F2 product scene must be UTF-8");

    let error = SceneAsset::from_project_toml_str(document, |reference| {
        let project = reference
            .project_ref()
            .expect("F2 scene references must use persisted project references");
        if project.path_hint().as_str() == missing_path_hint {
            Err(ReferenceResolutionError::MissingGuid {
                guid: project.guid(),
            })
        } else {
            Ok(resolve_template_project_reference(reference))
        }
    })
    .expect_err("F2 scene loading must reject an unresolved persisted project reference");

    assert!(matches!(
        error,
        ProjectDocumentError::Reference(ReferenceResolutionError::MissingGuid { .. })
    ));
}

fn assert_template_project_reference(
    reference: &toml::Value,
    expected_guid: &str,
    expected_path_hint: &str,
) {
    let reference = reference
        .as_table()
        .expect("F2 persisted asset reference must be a table");
    assert_eq!(
        reference.get("kind").and_then(toml::Value::as_str),
        Some("project")
    );
    assert_eq!(
        reference.get("guid").and_then(toml::Value::as_str),
        Some(expected_guid)
    );
    assert_eq!(
        reference.get("path_hint").and_then(toml::Value::as_str),
        Some(expected_path_hint)
    );
}

fn resolve_template_project_reference(
    reference: &zircon_runtime_interface::project::PersistedAssetReference,
) -> AssetReference {
    let project = reference
        .project_ref()
        .expect("F2 template references must use persisted project references");
    let relative_path = project
        .path_hint()
        .as_str()
        .strip_prefix("assets/")
        .expect("F2 template project references must be rooted below assets/");
    // Compound assets persist their validated `.zmeta` source while resolving to its logical URI.
    let relative_path = relative_path
        .strip_suffix(".zmeta")
        .unwrap_or(relative_path);
    let locator =
        AssetUri::parse(format!("res://{relative_path}")).expect("F2 project reference locator");
    AssetReference::new(project.guid(), locator)
}

fn renderable_empty_scene_document() -> Vec<u8> {
    renderable_empty_template_entry("assets/scenes/main.scene.toml")
}

fn renderable_empty_template_entry(path: &str) -> Vec<u8> {
    render_project_template(ProjectTemplateId::RenderableEmpty, "F2 Scene Contract")
        .expect("render F2 product template")
        .entries
        .into_iter()
        .find(|entry| entry.path.as_str() == path)
        .unwrap_or_else(|| panic!("F2 product template entry is missing: {path}"))
        .bytes
}
