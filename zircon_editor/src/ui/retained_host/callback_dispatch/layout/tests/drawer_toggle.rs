use super::drawer_tab_switch_reuses_layout;
use crate::ui::workbench::layout::ActivityDrawerMode;

#[test]
fn switching_tabs_in_an_open_drawer_reuses_layout() {
    assert!(drawer_tab_switch_reuses_layout(
        ActivityDrawerMode::Pinned,
        false,
        true,
    ));
    assert!(drawer_tab_switch_reuses_layout(
        ActivityDrawerMode::AutoHide,
        false,
        true,
    ));
}

#[test]
fn switching_between_drawers_in_an_expanded_region_reuses_layout() {
    assert!(drawer_tab_switch_reuses_layout(
        ActivityDrawerMode::Collapsed,
        false,
        true,
    ));
}

#[test]
fn collapse_and_reopen_keep_the_full_layout_path() {
    assert!(!drawer_tab_switch_reuses_layout(
        ActivityDrawerMode::Pinned,
        true,
        true,
    ));
    assert!(!drawer_tab_switch_reuses_layout(
        ActivityDrawerMode::Collapsed,
        false,
        false,
    ));
}
