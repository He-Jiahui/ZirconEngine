#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum WorkbenchMenuPointerRoute {
    MenuButton(usize),
    MenuItem {
        menu_index: usize,
        item_index: usize,
        action_id: String,
    },
    PopupSurface(usize),
    DismissOverlay,
}
