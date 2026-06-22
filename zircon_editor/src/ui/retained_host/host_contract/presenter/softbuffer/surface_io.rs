mod copy;
mod damage;
mod size;

pub(in crate::ui::retained_host::host_contract) use self::copy::copy_rgba_to_softbuffer;
pub(in crate::ui::retained_host::host_contract) use self::damage::{
    damage_pixel_count, pixel_bounds, softbuffer_damage_rect,
};
pub(in crate::ui::retained_host::host_contract) use self::size::{
    clamp_size, current_window_size, resize_surface,
};
