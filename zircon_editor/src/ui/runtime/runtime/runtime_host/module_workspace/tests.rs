use super::*;

#[test]
fn canonical_core_and_animation_workspaces_resolve_to_physical_asset_paths() {
    let asset_root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("assets");
    for path in [
        "ui/editor/modules/core/gameplay/ability_workspace.zui",
        "ui/editor/modules/core/gameplay/effect_workspace.zui",
        "ui/editor/animation/extension_blend_space_workspace.zui",
    ] {
        let uri = format!("res://{path}");
        let relative = workspace_asset_relative_path(&uri)
            .expect("current workspace owner should be accepted");
        assert_eq!(relative.as_path(), std::path::Path::new(path));
        assert!(asset_root.join(relative.as_path()).is_file());
    }
}

#[test]
fn unowned_and_traversing_workspace_uris_are_rejected() {
    for uri in [
        "plugin://example/ui/editor/animation/extension_blend_space_workspace.zui",
        "res://ui/editor/unreal/components/menu_popups.zui",
        "res://ui/editor/modules/core_other/gameplay/ability_workspace.zui",
        "res://ui/editor/components/workbench/modules/core/gameplay/ability_workspace.zui",
        "res://ui/editor/animation/../modules/core/gameplay/ability_workspace.zui",
        "res://ui/editor/modules/core//gameplay/ability_workspace.zui",
        "res://ui/editor/animation/./extension_blend_space_workspace.zui",
        "res://ui/editor/animation/..\\extension_blend_space_workspace.zui",
    ] {
        assert!(
            matches!(
                workspace_asset_relative_path(uri),
                Err(UiV2AssetError::InvalidDocument { .. })
            ),
            "unowned or invalid workspace URI must be rejected: {uri}",
        );
    }
}
