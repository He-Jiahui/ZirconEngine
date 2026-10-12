#[path = "chrome_damage/entry.rs"]
mod entry;
#[path = "chrome_damage/floating.rs"]
mod floating;
#[path = "chrome_damage/host_page.rs"]
mod host_page;
#[path = "chrome_damage/route.rs"]
mod route;
#[path = "chrome_damage/union.rs"]
mod union;

pub(in crate::ui::retained_host::host_contract) use self::entry::chrome_press_damage_frame;
