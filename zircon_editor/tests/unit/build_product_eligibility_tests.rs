use super::{editor_catalog_entry_from_manifest, EditorCatalogEntry};

#[test]
fn product_catalog_rejects_explicit_non_string_package_roles() {
    for role in [
        "false",
        "42",
        "3.14",
        "[\"test_fixture\"]",
        "{ kind = \"test_fixture\" }",
        "2026-09-27",
    ] {
        let error = match catalog_entry_with_role(Some(role)) {
            Err(error) => error,
            Ok(_) => panic!("explicit non-string role {role} must fail closed"),
        };
        assert!(error.to_string().contains("package_role must be a string"));
    }
}

#[test]
fn product_catalog_defaults_only_absent_roles_and_preserves_typed_eligibility() {
    for role in [None, Some("\"production\""), Some("\"developer_tool\"")] {
        let entry = catalog_entry_with_role(role)
            .expect("eligible role should parse")
            .expect("eligible role should produce a catalog entry");
        assert_eq!(entry.package_id, "role-probe");
    }
    for role in ["\"sample\"", "\"test_fixture\""] {
        assert!(catalog_entry_with_role(Some(role))
            .expect("carrier role should parse")
            .is_none());
    }
    for role in ["\"preview\"", "\"\"", "\" sample \""] {
        assert!(catalog_entry_with_role(Some(role)).is_err());
    }
}

fn catalog_entry_with_role(
    role: Option<&str>,
) -> Result<Option<EditorCatalogEntry>, Box<dyn std::error::Error>> {
    let path = std::env::temp_dir().join(format!(
        "zircon-editor-catalog-role-{}-{}.toml",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("system time after epoch")
            .as_nanos()
    ));
    let role_field = role
        .map(|value| format!("package_role = {value}\n"))
        .unwrap_or_default();
    std::fs::write(
        &path,
        format!(
            "id = \"role-probe\"\ndisplay_name = \"Role Probe\"\n{role_field}[[modules]]\nkind = \"editor\"\ncrate_name = \"role_probe_editor\"\n"
        ),
    )
    .expect("write role fixture");
    let result = editor_catalog_entry_from_manifest(&path);
    std::fs::remove_file(&path).expect("remove role fixture");
    result
}

#[test]
fn product_catalog_excludes_samples_and_test_fixtures() {
    let repository = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("editor crate should have repository parent");
    for relative in [
        "zircon_plugins/plugin_sdk_examples/plugin.toml",
        "zircon_plugins/native_dynamic_fixture/plugin.toml",
        "zircon_plugins/editor_contribution_fixture/plugin.toml",
    ] {
        let path = repository.join(relative);
        assert!(
            editor_catalog_entry_from_manifest(&path)
                .expect("carrier manifest should parse")
                .is_none(),
            "{} must stay out of product inventory",
            path.display()
        );
    }
}

#[test]
fn product_catalog_keeps_production_editor_packages() {
    let repository = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("editor crate should have repository parent");
    let path = repository.join("zircon_plugins/navigation/plugin.toml");
    let entry = editor_catalog_entry_from_manifest(&path)
        .expect("production manifest should parse")
        .expect("production editor package should be eligible");

    assert_eq!(entry.package_id, "navigation");
}
