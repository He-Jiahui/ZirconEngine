use super::*;

#[test]
fn material_editor_foundation_catalog_stays_folder_backed_by_family() {
    let catalog_root = Path::new(env!("CARGO_MANIFEST_DIR")).join("src/ui/component/catalog");
    assert!(
        !catalog_root.join("material_foundation.rs").exists(),
        "Material foundation catalog should stay split by component family"
    );

    let foundation_root = catalog_root.join("material_foundation");
    let expected_modules = [
        "mod.rs",
        "shared.rs",
        "button_inputs.rs",
        "inputs.rs",
        "selection_inputs.rs",
        "text_inputs.rs",
        "form_controls.rs",
        "lab_subcomponents.rs",
        "data_display.rs",
        "data_display_editor.rs",
        "data_display_subcomponents.rs",
        "data_display_table.rs",
        "data_display_visuals.rs",
        "feedback.rs",
        "feedback_editor_overlays.rs",
        "surfaces.rs",
        "navigation.rs",
        "navigation_subcomponents.rs",
        "navigation_secondary.rs",
        "navigation_editor.rs",
        "layout_mui.rs",
        "layout.rs",
        "layout_utilities.rs",
        "layout_transitions.rs",
        "layout_editor.rs",
        "mui_x.rs",
        "surface_subcomponents.rs",
    ];
    let actual_modules = fs::read_dir(&foundation_root)
        .unwrap_or_else(|error| {
            panic!(
                "Material foundation catalog directory is readable at {}: {error}",
                foundation_root.display()
            )
        })
        .map(|entry| {
            entry
                .expect("Material foundation module entry is readable")
                .file_name()
                .to_string_lossy()
                .to_string()
        })
        .collect::<BTreeSet<_>>();

    assert_eq!(
        actual_modules,
        expected_modules
            .iter()
            .copied()
            .map(str::to_string)
            .collect::<BTreeSet<_>>(),
        "Material foundation modules should remain grouped by the planned MUI families"
    );

    let mod_source = fs::read_to_string(foundation_root.join("mod.rs"))
        .expect("Material foundation mod.rs is readable");
    for module in expected_modules
        .iter()
        .copied()
        .filter(|module| *module != "mod.rs")
    {
        let stem = module
            .strip_suffix(".rs")
            .expect("expected Rust module file");
        assert!(
            mod_source.contains(&format!("mod {stem};")),
            "Material foundation mod.rs should declare `{stem}`"
        );
        let source = fs::read_to_string(foundation_root.join(module))
            .unwrap_or_else(|error| panic!("{module} is readable: {error}"));
        if stem != "shared" {
            assert!(
                source.lines().count() <= 300,
                "{module} should stay below the split-module size budget"
            );
            assert!(
                mod_source.contains(&format!("descriptors.extend({stem}::descriptors());")),
                "Material foundation registry should include `{stem}` descriptors"
            );
        }
    }
}
