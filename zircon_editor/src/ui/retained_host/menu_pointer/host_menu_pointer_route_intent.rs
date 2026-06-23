#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum HostMenuPointerRouteIntent {
    MenuButton(usize),
    SubmenuBranch {
        menu_index: usize,
        item_index: usize,
        item_path: Vec<usize>,
    },
    MenuItem {
        menu_index: usize,
        item_index: usize,
        item_path: Vec<usize>,
        action_id: String,
    },
    PopupSurface(usize),
    DismissOverlay,
}
