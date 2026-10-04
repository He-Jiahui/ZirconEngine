use super::RuntimePluginCatalog;

#[test]
fn catalog_constructors_do_not_rebuild_after_each_registration() {
    let source = include_str!("../constructors.rs");
    let incremental_register = ["catalog", ".register("].concat();
    assert!(!source.contains(&incremental_register));
}

#[test]
fn builtin_catalog_borrows_one_immutable_generation_without_cloning_rows() {
    let first = RuntimePluginCatalog::builtin();
    let second = RuntimePluginCatalog::builtin();

    assert!(std::ptr::eq(first, second));

    let constructor_source = include_str!("../constructors.rs")
        .split_once("#[cfg(test)]")
        .expect("constructor production section should precede tests")
        .0;
    assert!(constructor_source.contains("pub fn builtin() -> &'static Self"));
    assert!(!constructor_source.contains("builtin_shared().clone()"));

    let access_source = include_str!("../../access.rs");
    assert!(access_source.contains("impl ExactSizeIterator<Item = &PluginPackageManifest> + '_"));
    assert!(!access_source.contains("registration.package_manifest.clone()"));
}
