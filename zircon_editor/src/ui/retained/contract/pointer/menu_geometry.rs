#[path = "menu_geometry/bar.rs"]
mod bar;
#[path = "menu_geometry/damage.rs"]
mod damage;
#[path = "menu_geometry/frames.rs"]
mod frames;
#[path = "menu_geometry/popup.rs"]
mod popup;

pub(in crate::ui::retained_host::host_contract) use bar::menu_handles_point_with_state;
pub(in crate::ui::retained_host::host_contract) use damage::menu_damage_frame_with_state;
pub(in crate::ui::retained_host::host_contract) use popup::menu_popup_handles_point_with_state;
