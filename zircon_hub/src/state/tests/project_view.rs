use super::{ProjectFilterMode, ProjectSortMode, ProjectSubpage, ProjectViewMode};

#[test]
fn project_filter_mode_cycles_through_supported_modes() {
    assert_eq!(ProjectFilterMode::All.next(), ProjectFilterMode::Existing);
    assert_eq!(
        ProjectFilterMode::Existing.next(),
        ProjectFilterMode::Missing
    );
    assert_eq!(ProjectFilterMode::Missing.next(), ProjectFilterMode::All);
    assert_eq!(
        ProjectFilterMode::from_id("available"),
        Some(ProjectFilterMode::Existing)
    );
}

#[test]
fn project_sort_mode_cycles_between_supported_modes() {
    assert_eq!(ProjectSortMode::LastModified.next(), ProjectSortMode::Name);
    assert_eq!(ProjectSortMode::Name.next(), ProjectSortMode::LastModified);
}

#[test]
fn project_view_mode_parses_ui_ids() {
    assert_eq!(
        ProjectViewMode::from_id("grid"),
        Some(ProjectViewMode::Grid)
    );
    assert_eq!(
        ProjectViewMode::from_id("TABLE"),
        Some(ProjectViewMode::List)
    );
    assert_eq!(ProjectViewMode::from_id("unknown"), None);
}

#[test]
fn project_subpage_parses_internal_page_ids() {
    assert_eq!(
        ProjectSubpage::from_id("new-project"),
        Some(ProjectSubpage::NewProject)
    );
    assert_eq!(ProjectSubpage::ProjectBrowser.id(), "project-browser");
    assert_eq!(ProjectSubpage::from_id("missing"), None);
}
