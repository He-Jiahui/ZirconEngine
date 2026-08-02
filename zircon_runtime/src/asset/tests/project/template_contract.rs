use crate::asset::{
    AssetReference, AssetUri, ProjectDocumentError, ReferenceResolutionError, SceneAsset,
    SceneMobilityAsset,
};
use zircon_runtime_interface::project::{ProjectTemplateId, render_project_template};

#[test]
fn renderable_empty_template_parses_through_runtime_scene_schema_with_project_references() {
    let document = renderable_empty_scene_document();
    let document = std::str::from_utf8(&document).expect("F2 product scene must be UTF-8");

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
    assert!(camera.camera.is_some());

    let sun = scene
        .entities
        .iter()
        .find(|entity| entity.name == "Sun")
        .expect("F2 Sun entity");
    assert_eq!(sun.mobility, SceneMobilityAsset::Static);
    assert!(sun.directional_light.is_some());

    let cube = scene
        .entities
        .iter()
        .find(|entity| entity.name == "Cube")
        .expect("F2 Cube entity");
    assert_eq!(cube.mobility, SceneMobilityAsset::Static);
    assert!(cube.transform.scale.iter().all(|scale| *scale != 0.0));
    let mesh = cube.mesh.as_ref().expect("F2 Cube mesh binding");
    assert_eq!(
        mesh.model.locator,
        AssetUri::parse("res://models/cube.obj").expect("F2 cube model locator")
    );
    assert_eq!(
        mesh.material.locator,
        AssetUri::parse("res://materials/default.zmaterial").expect("F2 material locator")
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

fn resolve_template_project_reference(
    reference: &zircon_runtime_interface::project::PersistedAssetReference,
) -> AssetReference {
    let project = reference
        .project_ref()
        .expect("F2 scene references must use persisted project references");
    let relative_path = project
        .path_hint()
        .as_str()
        .strip_prefix("assets/")
        .expect("F2 project references must be rooted below assets/");
    let locator =
        AssetUri::parse(format!("res://{relative_path}")).expect("F2 project reference locator");
    AssetReference::new(project.guid(), locator)
}

fn renderable_empty_scene_document() -> Vec<u8> {
    render_project_template(ProjectTemplateId::RenderableEmpty, "F2 Scene Contract")
        .expect("render F2 product template")
        .entries
        .into_iter()
        .find(|entry| entry.path.as_str() == "assets/scenes/main.scene.toml")
        .expect("F2 product template default scene")
        .bytes
}
