use crate::ui::retained_host::primitives::SharedString;

pub(in crate::ui::retained_host::host_contract) enum ChromePointerRoute {
    ActivityRail {
        side: SharedString,
        local_x: f32,
        local_y: f32,
    },
    HostPageTab {
        index: usize,
        tab_x: f32,
        tab_width: f32,
        local_x: f32,
        local_y: f32,
    },
    HostPageOverflow {
        tab_x: f32,
        tab_width: f32,
        local_x: f32,
        local_y: f32,
    },
    DocumentTab {
        surface_key: SharedString,
        index: usize,
        tab_x: f32,
        tab_width: f32,
        local_x: f32,
        local_y: f32,
        close: bool,
    },
    DrawerHeaderTab {
        surface_key: SharedString,
        index: usize,
        tab_x: f32,
        tab_width: f32,
        local_x: f32,
        local_y: f32,
    },
    FloatingWindowHeader {
        window_id: SharedString,
    },
    Resize,
}
