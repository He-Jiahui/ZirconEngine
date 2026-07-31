mod colors;
mod draw;

pub(in crate::ui::retained_host::host_contract) use draw::{
    draw_debug_reflector_overlay, draw_debug_reflector_overlay_iter,
};

#[cfg(test)]
#[path = "paint_debug_reflector_overlay_tests.rs"]
mod tests;
