#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum HostMenuPointerRoute {
    MenuButton(usize),
    SubmenuBranch {
        menu_index: usize,
        item_index: usize,
    },
    MenuItem {
        menu_index: usize,
        item_index: usize,
        action_id: String,
    },
    PopupSurface(usize),
    DismissOverlay,
}
