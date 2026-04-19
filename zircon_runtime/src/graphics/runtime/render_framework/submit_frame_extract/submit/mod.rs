mod build_runtime_frame;
mod collect_gpu_completions;
mod release_previous_history;
mod resolve_history_handle;
mod submit;
mod submit_runtime_frame;

pub(in crate::graphics::runtime::render_framework) use submit::submit_frame_extract;
pub(in crate::graphics::runtime::render_framework) use submit_runtime_frame::submit_runtime_frame;
