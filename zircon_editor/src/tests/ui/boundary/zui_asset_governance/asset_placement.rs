use super::support::{
    collect_zui_files, editor_asset_root, is_component_directory_path, load_zui_document,
    pascal_case_file_stem, resource_locator_for_path, runtime_asset_root,
};

#[test]
fn production_zui_component_assets_live_in_component_directories() {
    let asset_roots = [editor_asset_root(), runtime_asset_root()];
    let mut checked_assets = 0usize;
    let mut component_directory_assets = 0usize;
    let mut offenders = Vec::new();

    for asset_root in &asset_roots {
        for path in collect_zui_files(&asset_root.join("ui")) {
            checked_assets += 1;
            if is_component_directory_path(asset_root, &path) {
                component_directory_assets += 1;
                continue;
            }

            offenders.push(format!(
                "{} is outside a component directory",
                path.display()
            ));
        }
    }

    assert!(
        checked_assets > 0,
        "production asset roots should contain .zui component assets"
    );
    assert!(
        component_directory_assets > 0,
        "production .zui assets should primarily live under component directories"
    );
    assert!(
        offenders.is_empty(),
        "production .zui component assets must live under component directories: {offenders:#?}"
    );
}

#[test]
fn editor_workbench_zui_assets_are_grouped_by_functional_component_folder() {
    let editor_root = editor_asset_root();
    let component_root = editor_root.join("ui/editor/components");
    let allowed_folders = [
        "res://ui/editor/components/showcase/",
        "res://ui/editor/components/workbench/primitives/inputs/",
        "res://ui/editor/components/workbench/primitives/data/",
        "res://ui/editor/components/workbench/primitives/feedback/",
        "res://ui/editor/components/workbench/primitives/chrome/",
        "res://ui/editor/components/workbench/composites/animation/",
        "res://ui/editor/components/workbench/composites/chrome/",
        "res://ui/editor/components/workbench/composites/feedback/",
        "res://ui/editor/components/workbench/composites/inputs/",
        "res://ui/editor/components/workbench/floating/",
        "res://ui/editor/components/workbench/shell/",
        "res://ui/editor/components/workbench/modules/core/ai/",
        "res://ui/editor/components/workbench/modules/core/assets/",
        "res://ui/editor/components/workbench/modules/core/gameplay/",
        "res://ui/editor/components/workbench/modules/core/index/",
        "res://ui/editor/components/workbench/modules/core/rendering/",
        "res://ui/editor/components/workbench/modules/core/ui/",
        "res://ui/editor/components/workbench/modules/extensions/animation/",
        "res://ui/editor/components/workbench/modules/extensions/data/",
        "res://ui/editor/components/workbench/modules/extensions/diagnostics/",
        "res://ui/editor/components/workbench/modules/extensions/gameplay/",
        "res://ui/editor/components/workbench/modules/extensions/index/",
        "res://ui/editor/components/workbench/modules/extensions/multiplayer/",
        "res://ui/editor/components/workbench/modules/extensions/production/",
        "res://ui/editor/components/workbench/modules/extensions/rendering/",
        "res://ui/editor/components/workbench/modules/extensions/simulation/",
        "res://ui/editor/components/workbench/modules/extensions/ui/",
        "res://ui/editor/components/workbench/modules/extensions/world/",
        "res://ui/editor/components/workbench/modules/generated/",
    ];
    let mut checked_assets = 0usize;
    let mut offenders = Vec::new();

    for path in collect_zui_files(&component_root) {
        checked_assets += 1;
        let locator = resource_locator_for_path(&editor_root, &path);
        if !allowed_folders
            .iter()
            .any(|folder| locator.starts_with(folder))
        {
            offenders.push(format!(
                "{locator} must live under a functional Workbench component folder"
            ));
        }
    }

    assert!(
        checked_assets >= 98,
        "editor Workbench component tree should include the current showcase, primitive, shell, module, extension, and generated .zui asset set"
    );
    assert!(
        offenders.is_empty(),
        "editor Workbench .zui assets must not be added flat under components/ or outside the functional folder taxonomy: {offenders:#?}"
    );
}

#[test]
fn editor_material_zui_assets_are_grouped_by_functional_component_folder() {
    let editor_root = editor_asset_root();
    let component_root = editor_root.join("ui/editor/material_components");
    let allowed_folders = [
        "res://ui/editor/material_components/data_display/",
        "res://ui/editor/material_components/feedback/",
        "res://ui/editor/material_components/inputs/",
        "res://ui/editor/material_components/layout/",
        "res://ui/editor/material_components/mui_x/",
        "res://ui/editor/material_components/navigation/",
        "res://ui/editor/material_components/surfaces/",
        "res://ui/editor/material_components/utils_lab/",
    ];
    let mut checked_assets = 0usize;
    let mut offenders = Vec::new();

    for path in collect_zui_files(&component_root) {
        checked_assets += 1;
        let locator = resource_locator_for_path(&editor_root, &path);
        if !allowed_folders
            .iter()
            .any(|folder| locator.starts_with(folder))
        {
            offenders.push(format!(
                "{locator} must live under a functional Material/MUI component folder"
            ));
        }
    }

    assert!(
        checked_assets >= 74,
        "editor Material/MUI component tree should include the current classified .zui prototype set"
    );
    assert!(
        offenders.is_empty(),
        "editor Material/MUI .zui assets must not be added flat under material_components/ or outside the functional folder taxonomy: {offenders:#?}"
    );
}

#[test]
fn production_zui_component_names_match_file_stems() {
    let asset_roots = [editor_asset_root(), runtime_asset_root()];
    let mut checked_assets = 0usize;
    let mut prototype_named_assets = 0usize;
    let mut offenders = Vec::new();

    for asset_root in &asset_roots {
        for path in collect_zui_files(&asset_root.join("ui")) {
            checked_assets += 1;
            let expected_name = pascal_case_file_stem(&path);
            let expected_prototype_name = format!("{expected_name}Prototype");
            let document = load_zui_document(&path);
            let component_name = document
                .components
                .keys()
                .next()
                .expect("UiZuiAssetLoader validates exactly one component");

            if component_name == &expected_prototype_name {
                prototype_named_assets += 1;
                continue;
            }
            if component_name != &expected_name {
                offenders.push(format!(
                    "{} declares component `{}` but expected `{}` or `{}`",
                    path.display(),
                    component_name,
                    expected_name,
                    expected_prototype_name
                ));
            }
        }
    }

    assert!(
        checked_assets > 0,
        "production asset roots should contain .zui component assets"
    );
    assert!(
        prototype_named_assets > 0,
        "production .zui assets should include explicit Prototype component names for lab/showcase prototypes"
    );
    assert!(
        offenders.is_empty(),
        "production .zui component names must match the file stem PascalCase, with an optional Prototype suffix for authoring prototypes: {offenders:#?}"
    );
}
