#[path = "../../../../post/cluster_dimensions/buffer_bytes_for_size.rs"]
mod cluster_buffer_bytes_for_size;
#[path = "for_size.rs"]
mod cluster_dimensions_for_size;

pub(crate) use cluster_buffer_bytes_for_size::cluster_buffer_bytes_for_size;
pub(crate) use cluster_dimensions_for_size::cluster_dimensions_for_size;
