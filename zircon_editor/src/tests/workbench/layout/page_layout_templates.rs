use std::collections::BTreeSet;

use crate::ui::workbench::autolayout::{EditorRegion, EditorRegionRole};
use crate::ui::workbench::layout::{ActivityDrawerMode, ActivityDrawerSlot, MainPageId};
use crate::ui::workbench::{CenterSplitLayout, LayoutPresetName, PageLayoutTemplate};

const PAGE_TEMPLATES_ASSET: &str =
    include_str!("../../../../assets/ui/editor/layout/page_templates.toml");

#[test]
fn built_in_page_templates_cover_thirteen_editor_pages() {
    let templates = PageLayoutTemplate::builtin_templates();
    let page_ids = templates
        .iter()
        .map(|template| template.page.clone())
        .collect::<Vec<_>>();

    assert_eq!(templates.len(), 13);
    assert_eq!(
        page_ids,
        expected_page_names()
            .into_iter()
            .map(MainPageId::new)
            .collect::<Vec<_>>()
    );
}

#[test]
fn built_in_page_templates_keep_region_roles_and_state_profiles_valid() {
    for template in PageLayoutTemplate::builtin_templates() {
        assert!(
            template.has_region_role(EditorRegion::Center, EditorRegionRole::CenterDocument),
            "template {:?} must declare center document content",
            template.page
        );
        assert_eq!(
            template.default_drawer_states.len(),
            ActivityDrawerSlot::ALL.len()
        );

        let drawer_slots = template
            .default_drawer_states
            .iter()
            .map(|state| state.slot)
            .collect::<BTreeSet<_>>();
        assert_eq!(
            drawer_slots,
            ActivityDrawerSlot::ALL.into_iter().collect::<BTreeSet<_>>()
        );

        for fill in &template.region_fills {
            assert_eq!(fill.role, expected_role(fill.region));
        }

        match template.default_center_split {
            CenterSplitLayout::SingleDocument => {}
            CenterSplitLayout::Split { panes, .. } => assert!(panes >= 2),
        }
    }
}

#[test]
fn page_templates_assign_focus_review_and_debug_state_profiles() {
    let templates = PageLayoutTemplate::builtin_templates();
    let game = templates
        .iter()
        .find(|template| template.page == MainPageId::new("game"))
        .expect("game template");
    assert_eq!(game.default_preset, LayoutPresetName::Focus);
    assert!(game
        .default_drawer_states
        .iter()
        .all(|state| state.mode == ActivityDrawerMode::Collapsed));

    let diagnostics = templates
        .iter()
        .find(|template| template.page == MainPageId::new("runtime_diagnostics"))
        .expect("runtime diagnostics template");
    assert_eq!(diagnostics.default_preset, LayoutPresetName::Debug);
    assert!(diagnostics.default_drawer_states.iter().any(|state| {
        state.slot == ActivityDrawerSlot::Bottom && state.mode == ActivityDrawerMode::Pinned
    }));

    let material = templates
        .iter()
        .find(|template| template.page == MainPageId::new("material"))
        .expect("material template");
    assert!(matches!(
        material.default_center_split,
        CenterSplitLayout::Split { panes: 2, .. }
    ));
}

#[test]
fn page_templates_asset_declares_same_pages_and_state_fields() {
    assert_eq!(PAGE_TEMPLATES_ASSET.matches("[[pages]]").count(), 13);
    for page in expected_page_names() {
        assert!(
            PAGE_TEMPLATES_ASSET.contains(&format!("page = \"{page}\"")),
            "asset must declare page `{page}`"
        );
    }
    assert_eq!(PAGE_TEMPLATES_ASSET.matches("center_split = ").count(), 13);
    assert_eq!(PAGE_TEMPLATES_ASSET.matches("drawer_modes = ").count(), 13);
    assert!(PAGE_TEMPLATES_ASSET.contains("default_preset = \"focus\""));
    assert!(PAGE_TEMPLATES_ASSET.contains("default_preset = \"review\""));
    assert!(PAGE_TEMPLATES_ASSET.contains("default_preset = \"debug\""));
}

fn expected_page_names() -> [&'static str; 13] {
    [
        "scene",
        "game",
        "material",
        "material_preview",
        "inspector",
        "prefab",
        "ui_designer",
        "ui_source",
        "animation_timeline",
        "animation_graph",
        "asset_browser",
        "console",
        "runtime_diagnostics",
    ]
}

fn expected_role(region: EditorRegion) -> EditorRegionRole {
    match region {
        EditorRegion::LeftTop => EditorRegionRole::PlacementTools,
        EditorRegion::LeftBottom => EditorRegionRole::ProjectTree,
        EditorRegion::Center => EditorRegionRole::CenterDocument,
        EditorRegion::RightTop => EditorRegionRole::HierarchyStructure,
        EditorRegion::RightBottom => EditorRegionRole::DetailInspector,
        EditorRegion::Bottom => EditorRegionRole::ConsoleDiagnosticsTimeline,
    }
}
