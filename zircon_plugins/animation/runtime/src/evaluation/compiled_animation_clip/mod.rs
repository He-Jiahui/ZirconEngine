//! 剪辑编译子模块的导出边界；公开结果供评估器缓存并按骨架修订失效。
mod compile;
mod compiled_animation_clip;

pub use compiled_animation_clip::CompiledAnimationClip;
