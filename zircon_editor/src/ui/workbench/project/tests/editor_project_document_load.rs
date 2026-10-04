#[test]
fn project_document_load_uses_only_the_activated_settings_authority_in_production() {
    let document_source = include_str!("../editor_project_document_load.rs").replace("\r\n", "\n");
    let host_source = include_str!("../../../host/project_access.rs").replace("\r\n", "\n");

    let test_helper_gate = document_source
        .find("    #[cfg(test)]\n    pub(crate) fn load_from_project_for_tests(")
        .expect("test-only direct project loader");
    let production_source = &document_source[..test_helper_gate];
    let retired_loader_signature = ["fn load_from_", "project("].concat();
    assert!(!production_source.contains(&retired_loader_signature));
    assert!(!production_source.contains("SettingsAuthority::with_defaults()"));
    assert!(production_source.contains("pub(crate) fn load_from_activated_project("));
    assert!(production_source.contains("world: if allows_scene_restore"));
    assert!(production_source.contains("Scene::load_scene_from_uri"));
    assert!(production_source.contains("Scene::new()"));

    let test_helper_source = &document_source[test_helper_gate..];
    assert!(test_helper_source.contains("SettingsAuthority::with_defaults()"));
    assert!(host_source.contains(
        "EditorProjectDocument::load_from_activated_project(\n            &project,\n            project_info,\n            self.settings.as_ref(),\n            allows_scene_restore,\n        )?;"
    ));
}
