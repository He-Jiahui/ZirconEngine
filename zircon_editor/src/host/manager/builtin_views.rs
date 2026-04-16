use crate::default_constraints_for_content;
use crate::layout::ActivityDrawerSlot;
use crate::view::{DockPolicy, PreferredHost, ViewDescriptor, ViewDescriptorId, ViewKind};
use crate::ViewContentKind;

use super::startup::welcome_view_descriptor;
use super::ui_asset_sessions::ui_asset_editor_view_descriptor;

pub(super) fn builtin_view_descriptors() -> Vec<ViewDescriptor> {
    vec![
        ViewDescriptor::new(
            ViewDescriptorId::new("editor.project"),
            ViewKind::ActivityView,
            "Project",
        )
        .with_preferred_drawer_slot(ActivityDrawerSlot::LeftTop)
        .with_default_constraints(default_constraints_for_content(ViewContentKind::Project))
        .with_icon_key("project"),
        ViewDescriptor::new(
            ViewDescriptorId::new("editor.hierarchy"),
            ViewKind::ActivityView,
            "Hierarchy",
        )
        .with_preferred_drawer_slot(ActivityDrawerSlot::LeftTop)
        .with_default_constraints(default_constraints_for_content(ViewContentKind::Hierarchy))
        .with_icon_key("hierarchy"),
        ViewDescriptor::new(
            ViewDescriptorId::new("editor.inspector"),
            ViewKind::ActivityView,
            "Inspector",
        )
        .with_preferred_drawer_slot(ActivityDrawerSlot::RightTop)
        .with_default_constraints(default_constraints_for_content(ViewContentKind::Inspector))
        .with_icon_key("inspector"),
        ViewDescriptor::new(
            ViewDescriptorId::new("editor.scene"),
            ViewKind::ActivityView,
            "Scene",
        )
        .with_dock_policy(DockPolicy::DrawerOrDocument)
        .with_preferred_host(PreferredHost::DocumentCenter)
        .with_default_constraints(default_constraints_for_content(ViewContentKind::Scene))
        .with_icon_key("scene"),
        ViewDescriptor::new(
            ViewDescriptorId::new("editor.game"),
            ViewKind::ActivityView,
            "Game",
        )
        .with_dock_policy(DockPolicy::DrawerOrDocument)
        .with_preferred_host(PreferredHost::DocumentCenter)
        .with_default_constraints(default_constraints_for_content(ViewContentKind::Game))
        .with_icon_key("game"),
        ViewDescriptor::new(
            ViewDescriptorId::new("editor.assets"),
            ViewKind::ActivityView,
            "Assets",
        )
        .with_preferred_drawer_slot(ActivityDrawerSlot::LeftTop)
        .with_default_constraints(default_constraints_for_content(ViewContentKind::Assets))
        .with_icon_key("assets"),
        ViewDescriptor::new(
            ViewDescriptorId::new("editor.console"),
            ViewKind::ActivityView,
            "Console",
        )
        .with_preferred_drawer_slot(ActivityDrawerSlot::BottomLeft)
        .with_default_constraints(default_constraints_for_content(ViewContentKind::Console))
        .with_icon_key("console"),
        ViewDescriptor::new(
            ViewDescriptorId::new("editor.prefab"),
            ViewKind::ActivityWindow,
            "Prefab Editor",
        )
        .with_multi_instance(true)
        .with_preferred_host(PreferredHost::DocumentCenter)
        .with_default_constraints(default_constraints_for_content(
            ViewContentKind::PrefabEditor,
        ))
        .with_icon_key("prefab"),
        ViewDescriptor::new(
            ViewDescriptorId::new("editor.asset_browser"),
            ViewKind::ActivityWindow,
            "Asset Browser",
        )
        .with_preferred_host(PreferredHost::ExclusiveMainPage)
        .with_default_constraints(default_constraints_for_content(
            ViewContentKind::AssetBrowser,
        ))
        .with_icon_key("asset-browser"),
        ui_asset_editor_view_descriptor(),
        welcome_view_descriptor(),
    ]
}
