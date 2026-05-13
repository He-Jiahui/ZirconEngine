use crate::ui::workbench::autolayout::default_constraints_for_content;
use crate::ui::workbench::snapshot::ViewContentKind;
use crate::ui::workbench::view::{
    ActivityWindowTemplateSpec, PreferredHost, ViewDescriptor, ViewDescriptorId, ViewKind,
};

pub(super) fn functional_window_view_descriptors() -> Vec<ViewDescriptor> {
    vec![
        ViewDescriptor::new(
            ViewDescriptorId::new("editor.scene_game_window"),
            ViewKind::ActivityWindow,
            "Scene/Game",
        )
        .with_preferred_host(PreferredHost::DocumentCenter)
        .with_default_constraints(default_constraints_for_content(ViewContentKind::Scene))
        .with_icon_key("scene-game-window"),
        ViewDescriptor::new(
            ViewDescriptorId::new("editor.prefab_editor_window"),
            ViewKind::ActivityWindow,
            "Prefab Editor",
        )
        .with_multi_instance(true)
        .with_preferred_host(PreferredHost::FloatingWindow)
        .with_default_constraints(default_constraints_for_content(
            ViewContentKind::PrefabEditor,
        ))
        .with_icon_key("prefab"),
        ViewDescriptor::new(
            ViewDescriptorId::new("editor.material_editor_window"),
            ViewKind::ActivityWindow,
            "Material Editor",
        )
        .with_multi_instance(true)
        .with_preferred_host(PreferredHost::FloatingWindow)
        .with_default_constraints(default_constraints_for_content(
            ViewContentKind::AssetBrowser,
        ))
        .with_icon_key("material-editor"),
        ViewDescriptor::new(
            ViewDescriptorId::new("editor.ui_asset_editor_window"),
            ViewKind::ActivityWindow,
            "UI Asset Editor",
        )
        .with_multi_instance(true)
        .with_preferred_host(PreferredHost::FloatingWindow)
        .with_default_constraints(default_constraints_for_content(
            ViewContentKind::UiAssetEditor,
        ))
        .with_activity_window_template(ActivityWindowTemplateSpec::new(
            "editor.window.ui_layout_editor",
        ))
        .with_icon_key("ui-asset-editor"),
        ViewDescriptor::new(
            ViewDescriptorId::new("editor.animation_editor_window"),
            ViewKind::ActivityWindow,
            "Animation Editor",
        )
        .with_multi_instance(true)
        .with_preferred_host(PreferredHost::FloatingWindow)
        .with_default_constraints(default_constraints_for_content(
            ViewContentKind::AnimationGraphEditor,
        ))
        .with_icon_key("animation-editor"),
        ViewDescriptor::new(
            ViewDescriptorId::new("editor.asset_browser_window"),
            ViewKind::ActivityWindow,
            "Asset Browser",
        )
        .with_preferred_host(PreferredHost::ExclusiveMainPage)
        .with_default_constraints(default_constraints_for_content(
            ViewContentKind::AssetBrowser,
        ))
        .with_activity_window_template(ActivityWindowTemplateSpec::new("editor.window.asset"))
        .with_icon_key("asset-browser"),
        ViewDescriptor::new(
            ViewDescriptorId::new("editor.diagnostics_window"),
            ViewKind::ActivityWindow,
            "Diagnostics",
        )
        .with_preferred_host(PreferredHost::ExclusiveMainPage)
        .with_default_constraints(default_constraints_for_content(
            ViewContentKind::RuntimeDiagnostics,
        ))
        .with_icon_key("diagnostics-window"),
    ]
}
