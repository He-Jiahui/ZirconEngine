//! 剪辑编译子模块的导出边界；公开结果供评估器缓存并按骨架修订失效。
#[path = "../compile.rs"]
mod compile;
#[path = "../compiled_animation_clip.rs"]
mod compiled_animation_clip;

pub use compiled_animation_clip::CompiledAnimationClip;
