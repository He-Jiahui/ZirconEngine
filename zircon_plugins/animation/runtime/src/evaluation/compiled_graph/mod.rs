//! 图编译和求值公共边界；宿主先编译源图与骨架，再以参数集取得剪辑贡献。
mod compile;
mod error;
mod evaluate;
mod types;

pub use compile::compile_animation_graph_runtime;
pub use error::AnimationGraphCompileError;
pub use types::{
    CompiledAnimationGraph, CompiledAnimationGraphEvaluation, CompiledGraphClipInstance,
};
