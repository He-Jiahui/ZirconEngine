#[path = "paint_debug_reflector_overlay/colors.rs"]
mod colors;
#[path = "paint_debug_reflector_overlay/draw.rs"]
mod draw;

pub(in crate::ui::retained_host::host_contract) use draw::{
    draw_debug_reflector_overlay, draw_debug_reflector_overlay_iter,
};

#[cfg(test)]
#[path = "tests/paint_debug_reflector_overlay.rs"]
mod tests;
