#[derive(Clone, Debug, PartialEq, Eq)]
pub(in crate::ui::slint_host::menu_pointer) enum WorkbenchMenuPointerTarget {
    MenuButton(usize),
    MenuItem {
        menu_index: usize,
        item_index: usize,
        action_id: String,
    },
    PopupSurface(usize),
    DismissOverlay,
}
