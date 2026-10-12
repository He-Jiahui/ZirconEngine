//! 动画运行时的剪辑、图、状态机和姿态评估导出层；帧入口通过 pipeline 消费这些构件。
#[path = "animation_clip_compile_error.rs"]
mod animation_clip_compile_error;
#[path = "../animation_evaluation_diagnostic.rs"]
mod animation_evaluation_diagnostic;
#[path = "clip_evaluator/mod.rs"]
mod clip_evaluator;
#[path = "compiled_animation_clip/mod.rs"]
mod compiled_animation_clip;
#[path = "compiled_clip_track.rs"]
mod compiled_clip_track;
#[path = "compiled_graph/mod.rs"]
mod compiled_graph;
#[path = "pipeline/mod.rs"]
mod pipeline;
#[path = "pose_blend_error.rs"]
mod pose_blend_error;
#[path = "pose_buffer/mod.rs"]
mod pose_buffer;
#[path = "pose_buffer_error.rs"]
mod pose_buffer_error;
#[path = "pose_pool.rs"]
mod pose_pool;
#[path = "skeleton_target_table.rs"]
mod skeleton_target_table;
#[path = "../state_machine_layer_diagnostic.rs"]
mod state_machine_layer_diagnostic;
#[path = "target_slot.rs"]
mod target_slot;
#[path = "target_table.rs"]
mod target_table;
#[path = "target_table_error.rs"]
mod target_table_error;

pub use animation_clip_compile_error::AnimationClipCompileError;
pub use animation_evaluation_diagnostic::AnimationEvaluationDiagnostic;
pub use clip_evaluator::{
    AnimationAssetRevision, AnimationChannelDataRole, AnimationClipEvaluator,
    AnimationClipEvaluatorStats, AnimationEvaluationError, AnimationTransformChannel,
};
pub use compiled_animation_clip::CompiledAnimationClip;
pub use compiled_clip_track::CompiledClipTrack;
pub use compiled_graph::{
    compile_animation_graph_runtime, AnimationGraphCompileError, CompiledAnimationGraph,
    CompiledAnimationGraphEvaluation, CompiledGraphClipInstance,
};
pub(crate) use pipeline::tick_animation_world;
pub use pipeline::{
    AnimationEvaluationPipeline, AnimationEvaluationProjectionStats, DirectClipWorkerStats,
    MAX_DIRECT_CLIP_WORKER_SHARDS,
};
pub use pose_blend_error::PoseBlendError;
pub use pose_buffer::{PoseBuffer, PoseLayer, PoseLayerBlendMode};
pub use pose_buffer_error::PoseBufferError;
pub use pose_pool::PosePool;
pub use skeleton_target_table::SkeletonTargetTable;
pub use state_machine_layer_diagnostic::{
    AnimationStateMachineLayerDiagnostic, AnimationStateMachineLayerError,
};
pub(crate) use target_slot::TargetSlot;
pub(crate) use target_table::TargetTable;
pub(crate) use target_table_error::TargetTableError;
