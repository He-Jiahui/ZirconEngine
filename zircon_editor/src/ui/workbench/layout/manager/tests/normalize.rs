use super::*;
use crate::ui::workbench::layout::{ActivityDrawerLayout, ActivityDrawerSlot};
use crate::ui::workbench::view::ViewInstanceId;

#[test]
fn drawer_selection_only_clones_when_repair_is_required() {
    let selected = ViewInstanceId::new("editor.selection#stable");
    let mut drawer = ActivityDrawerLayout::new(ActivityDrawerSlot::LeftTop);
    drawer.tab_stack.tabs.push(selected.clone());
    drawer.tab_stack.active_tab = Some(selected.clone());
    drawer.active_view = Some(selected.clone());
    let mut repairs = 0;

    normalize_drawer(&mut drawer, &mut repairs);

    assert_eq!(repairs, 0);
    assert_eq!(drawer.tab_stack.active_tab, Some(selected.clone()));
    assert_eq!(drawer.active_view, Some(selected.clone()));

    drawer.tab_stack.active_tab = Some(ViewInstanceId::new("editor.missing#tab"));
    drawer.active_view = Some(ViewInstanceId::new("editor.missing#view"));
    normalize_drawer(&mut drawer, &mut repairs);

    assert_eq!(repairs, 2);
    assert_eq!(drawer.tab_stack.active_tab, Some(selected.clone()));
    assert_eq!(drawer.active_view, Some(selected));
}
