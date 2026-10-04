use super::HubPage;

#[test]
fn hub_page_parses_known_navigation_ids() {
    assert_eq!(HubPage::from_id("projects"), Some(HubPage::Projects));
    assert_eq!(HubPage::from_id("EDITOR"), Some(HubPage::Editor));
    assert_eq!(HubPage::from_id("settings"), Some(HubPage::Settings));
    assert_eq!(HubPage::from_id("missing"), None);
}
