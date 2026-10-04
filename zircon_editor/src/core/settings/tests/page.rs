use super::SettingsPageDescriptor;

#[test]
fn settings_page_accepts_only_locale_neutral_presentation() {
    let descriptor = SettingsPageDescriptor::new(
        "plugin.fixture.settings",
        "fixture.editor",
        "plugin.fixture.label",
        "plugin.fixture.description",
        ["plugin.category.plugins", "plugin.category.fixture"],
    )
    .expect("locale-neutral page presentation should be accepted");

    assert_eq!(descriptor.localization_bundle_id(), "fixture.editor");
    assert_eq!(
        descriptor.category_keys().collect::<Vec<_>>(),
        ["plugin.category.plugins", "plugin.category.fixture"]
    );
    assert!(SettingsPageDescriptor::new(
        "plugin.fixture.legacy",
        "fixture.editor",
        "Legacy settings",
        "plugin.fixture.description",
        ["Plugins/Fixture"],
    )
    .is_err());
}
