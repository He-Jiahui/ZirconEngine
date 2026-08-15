mod draw;
mod raster;
mod recording;

pub(in crate::ui::retained_host::host_contract) use draw::{
    draw_gpu_image_clipped_with_resource_key, draw_rgba_image_clipped,
    draw_rgba_image_clipped_with_atlas, draw_rgba_image_clipped_with_resource_key,
    draw_shared_rgba_image_clipped_with_resource_key,
};

#[cfg(test)]
mod tests;
