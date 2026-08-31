//! Native command encoding split by execution domain.

mod compute;
mod compute_pass;
mod render_pass;
mod state;
mod texture_copy;

pub(crate) use compute::{
    encode_buffer_copy, encode_compute_dispatch, encode_compute_dispatch_indirect,
};
pub(crate) use compute_pass::encode_compute_pass;
pub(crate) use render_pass::encode_render_pass;
pub(crate) use state::{
    pop_debug_group, require_graphics_queue, validate_bind_group_slot, validate_debug_label,
    validate_index_buffer, validate_required_vertex_buffers, validate_vertex_buffer, BufferBinding,
    DebugGroupScope, EncoderState, IndexBufferBinding,
};
pub(crate) use texture_copy::{
    encode_buffer_to_texture_copy, encode_texture_to_buffer_copy, encode_texture_to_texture_copy,
};
