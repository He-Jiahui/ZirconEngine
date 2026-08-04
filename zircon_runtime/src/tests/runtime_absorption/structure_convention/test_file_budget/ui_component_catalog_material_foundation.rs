use super::*;

#[test]
fn runtime_15_ui_component_catalog_material_foundation_tests_are_folder_backed() {
    let parent = read_runtime_src("ui/tests/component_catalog/material_foundation/mod.rs");
    let editor_components =
        read_runtime_src("ui/tests/component_catalog/material_foundation/editor_components.rs");
    let folder_structure =
        read_runtime_src("ui/tests/component_catalog/material_foundation/folder_structure.rs");
    let mui_surface_overlay =
        read_runtime_src("ui/tests/component_catalog/material_foundation/mui_surface_overlay.rs");
    let mui_x_runtime =
        read_runtime_src("ui/tests/component_catalog/material_foundation/mui_x_runtime.rs");
    let planned_layers =
        read_runtime_src("ui/tests/component_catalog/material_foundation/planned_layers.rs");

    assert_contains_all(
        "UI Material foundation parent mounts folder-backed children",
        &parent,
        &[
            "mod editor_components;",
            "mod folder_structure;",
            "mod mui_surface_overlay;",
            "mod mui_x_runtime;",
            "mod planned_layers;",
            "fn assert_button_style_schema(",
            "fn assert_mui_web_customization_schema(",
        ],
    );
    assert_eq!(
        parent.matches("#[test]").count(),
        0,
        "ui/tests/component_catalog/material_foundation/mod.rs should only mount child test owners and shared helpers"
    );
    for moved_test in [
        "material_editor_foundation_catalog_covers_planned_component_layers",
        "material_editor_foundation_catalog_covers_editor_descriptor_contracts",
        "material_editor_foundation_catalog_covers_mui_surface_overlay_contracts",
        "material_editor_foundation_catalog_covers_mui_x_runtime_visibility_contracts",
        "material_editor_foundation_catalog_stays_folder_backed_by_family",
    ] {
        assert!(
            !parent.contains(moved_test),
            "moved UI Material foundation test `{moved_test}` should not return to the parent"
        );
    }

    assert_contains_all(
        "UI Material foundation planned-layers child owns inventory contracts",
        &planned_layers,
        &[
            "fn material_editor_foundation_catalog_covers_planned_component_layers",
            "UiComponentDescriptorRegistry::material_editor_foundation",
            "assert_mui_web_customization_schema",
        ],
    );
    assert_contains_all(
        "UI Material foundation editor child owns editor descriptor contracts",
        &editor_components,
        &[
            "fn material_editor_foundation_catalog_covers_editor_descriptor_contracts",
            "TextField",
            "ViewportHost",
            "WorkbenchShell",
        ],
    );
    assert_contains_all(
        "UI Material foundation MUI surface child owns overlay/feedback contracts",
        &mui_surface_overlay,
        &[
            "fn material_editor_foundation_catalog_covers_mui_surface_overlay_contracts",
            "surface_variant",
            "SnackbarContent",
            "surfaces::assert_descriptors",
        ],
    );
    assert_contains_all(
        "UI Material foundation MUI X child owns runtime visibility contracts",
        &mui_x_runtime,
        &[
            "fn material_editor_foundation_catalog_covers_mui_x_runtime_visibility_contracts",
            "MaterialTreeView",
            "DataGrid",
            "runtime_visible_ids",
        ],
    );
    assert_contains_all(
        "UI Material foundation folder child owns folder structure guard",
        &folder_structure,
        &["fn material_editor_foundation_catalog_stays_folder_backed_by_family"],
    );

    let child_test_total = [
        editor_components.as_str(),
        folder_structure.as_str(),
        mui_surface_overlay.as_str(),
        mui_x_runtime.as_str(),
        planned_layers.as_str(),
    ]
    .into_iter()
    .map(|source| source.matches("#[test]").count())
    .sum::<usize>();
    assert_eq!(
        child_test_total, 5,
        "UI Material foundation children should preserve the split catalog coverage"
    );

    for (path, source) in [
        (
            "ui/tests/component_catalog/material_foundation/mod.rs",
            parent.as_str(),
        ),
        (
            "ui/tests/component_catalog/material_foundation/editor_components.rs",
            editor_components.as_str(),
        ),
        (
            "ui/tests/component_catalog/material_foundation/folder_structure.rs",
            folder_structure.as_str(),
        ),
        (
            "ui/tests/component_catalog/material_foundation/mui_surface_overlay.rs",
            mui_surface_overlay.as_str(),
        ),
        (
            "ui/tests/component_catalog/material_foundation/mui_x_runtime.rs",
            mui_x_runtime.as_str(),
        ),
        (
            "ui/tests/component_catalog/material_foundation/planned_layers.rs",
            planned_layers.as_str(),
        ),
    ] {
        let line_count = source.lines().count();
        assert!(
            line_count < 800,
            "{path} should stay below the Runtime 15 test-file budget; got {line_count} lines"
        );
    }

    let runtime_15_plan =
        read_repo("docs/plans/zircon_runtime/runtime/15-code-structure-and-module-conventions.md");
    let runtime_index = read_repo("docs/plans/zircon_runtime/runtime/index.md");
    let review_findings = read_repo("docs/plans/engine-code-review-findings-2026-06.md");
    let structure_convention = read_repo("docs/plans/engine-code-structure-convention.md");
    let module_doc = read_repo("docs/zircon_runtime/structure/module-convention.md");
    let ui_doc = read_repo("docs/zircon_runtime/ui/architecture.md");
    for (label, source) in [
        ("Runtime 15 plan", runtime_15_plan.as_str()),
        ("Runtime index", runtime_index.as_str()),
        ("review findings", review_findings.as_str()),
        ("structure convention", structure_convention.as_str()),
        ("module convention doc", module_doc.as_str()),
        ("UI architecture doc", ui_doc.as_str()),
    ] {
        assert_contains_all(
            label,
            source,
            &[
                "Runtime 15 M3 UI Material foundation test folder split",
                "runtime_15_ui_component_catalog_material_foundation_tests_folder_split_static_passed_cargo_deferred",
                "ui/tests/component_catalog/material_foundation/mod.rs",
                "ui/tests/component_catalog/material_foundation/planned_layers.rs",
                "ui/tests/component_catalog/material_foundation/editor_components.rs",
                "runtime_15_ui_component_catalog_material_foundation_tests_are_folder_backed",
            ],
        );
    }
}
