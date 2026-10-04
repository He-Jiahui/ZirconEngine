use super::*;

#[test]
fn only_renderable_empty_is_enabled_for_creation() {
    assert_eq!(
        enabled_project_template_id("renderable-empty"),
        Some(ProjectTemplateId::RenderableEmpty)
    );
    assert_eq!(enabled_project_template_id("3d-scene"), None);
    assert_eq!(
        project_template_catalog()
            .iter()
            .filter(|template| template.enabled)
            .map(|template| template.id)
            .collect::<Vec<_>>(),
        vec!["renderable-empty"]
    );
}

#[test]
fn enabled_template_id_requires_the_canonical_spelling() {
    assert_eq!(enabled_project_template_id("  renderable-empty  "), None);
    assert_eq!(enabled_project_template_id("RENDERABLE-EMPTY"), None);
}

#[test]
fn enabled_template_id_match_keeps_non_ascii_case_strict() {
    assert_eq!(enabled_project_template_id("renderablE-empt\u{00dd}"), None);
}

#[test]
fn create_request_preserves_name_and_validates_launch_fields() {
    let request =
        CreateProjectRequest::new("My Game", "E:/Projects", ProjectTemplateId::RenderableEmpty);

    assert_eq!(request.project_name, "My Game");
    assert_eq!(
        request.target_root(),
        PathBuf::from("E:/Projects").join("My Game")
    );
    assert_eq!(request.validate_launch_fields(), Ok(()));

    let padded_name = CreateProjectRequest::new("  My Game  ", "E:/Projects", request.template);
    assert!(matches!(
        padded_name.validate_launch_fields(),
        Err(CreateProjectRequestError::ProjectName {
            source: ProjectNameError::SurroundingWhitespace { .. }
        })
    ));

    let missing_name = CreateProjectRequest::new("   ", "E:/Projects", request.template);
    assert_eq!(
        missing_name.validate_launch_fields(),
        Err(CreateProjectRequestError::ProjectName {
            source: ProjectNameError::Empty
        })
    );
    let missing_location = CreateProjectRequest::new("Game", "", request.template);
    assert_eq!(
        missing_location.validate_launch_fields(),
        Err(CreateProjectRequestError::MissingLocation)
    );
}

#[test]
fn create_request_rejects_unsafe_filename_components() {
    for name in ["..", "folder/Game", r"folder\Game", "NUL", "Game.", "Game "] {
        let request =
            CreateProjectRequest::new(name, "E:/Projects", ProjectTemplateId::RenderableEmpty);
        assert!(
            request.validate_launch_fields().is_err(),
            "accepted {name:?}"
        );
    }
}
