use super::ProjectTemplateId;

#[test]
fn parse_accepts_only_the_canonical_template_id() {
    assert_eq!(
        ProjectTemplateId::parse("renderable-empty"),
        Some(ProjectTemplateId::RenderableEmpty)
    );
    assert_eq!(ProjectTemplateId::parse("RENDERABLE-EMPTY"), None);
    assert_eq!(ProjectTemplateId::parse(" renderable-empty "), None);
}
