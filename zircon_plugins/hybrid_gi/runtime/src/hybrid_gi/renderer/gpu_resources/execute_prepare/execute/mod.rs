mod card_capture_shading;
mod collect_inputs;
mod copy_readbacks;
mod create_bind_group;
mod create_buffers;
mod dispatch;
mod dispatch_probe_trace_tiles;
mod execute;
mod hybrid_gi_prepare_execution_buffers;
mod hybrid_gi_prepare_execution_inputs;
mod material_capture_source;
mod queue_params;
mod voxel_clipmap_debug;

pub(in crate::hybrid_gi::renderer) use material_capture_source::{
    HybridGiMaterialCaptureSeed, HybridGiMaterialCaptureSource,
};
