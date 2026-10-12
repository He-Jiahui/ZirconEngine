//! 图编译和求值公共边界；宿主先编译源图与骨架，再以参数集取得剪辑贡献。
#[path = "compile.rs"]
mod compile;
#[path = "error.rs"]
mod error;
#[path = "evaluate.rs"]
mod evaluate;
#[path = "types.rs"]
mod types;

pub use compile::compile_animation_graph_runtime;
pub use error::AnimationGraphCompileError;
pub use types::{
    CompiledAnimationGraph, CompiledAnimationGraphEvaluation, CompiledGraphClipInstance,
};
