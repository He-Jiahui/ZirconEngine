use crate::ui::workbench::layout::MainPageId;

use super::{PreferredHost, ViewHost};

pub(super) fn preferred_host_to_view_host(preferred_host: PreferredHost) -> ViewHost {
    match preferred_host {
        PreferredHost::Drawer(slot) => ViewHost::Drawer(slot),
        PreferredHost::DocumentCenter => ViewHost::Document(MainPageId::workbench(), vec![]),
        PreferredHost::FloatingWindow => {
            ViewHost::FloatingWindow(MainPageId::new("floating"), vec![])
        }
        PreferredHost::ExclusiveMainPage => ViewHost::ExclusivePage(MainPageId::new("exclusive")),
    }
}
