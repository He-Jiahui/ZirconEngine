mod build_runtime_frame;
mod collect_gpu_completions;
mod release_previous_history;
mod resolve_history_handle;
mod submit;

pub(in crate::runtime::server) use submit::submit_frame_extract;
