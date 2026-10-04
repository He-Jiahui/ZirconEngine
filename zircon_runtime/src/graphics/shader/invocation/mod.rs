mod compiler;
mod diagnostic;
mod fullscreen_pass;
mod parameter_packer;
mod resource_bindings;

pub use compiler::{
    ComputeDispatchBuilder, ComputeDispatchPlan, ComputeKernelRef, ComputePipelineCacheKey,
    ShaderAbiBinding, ShaderDispatchExtent, COMPUTE_SHADER_FIRST_RESOURCE_BINDING,
    COMPUTE_SHADER_PARAMS_BINDING, COMPUTE_SHADER_RESOURCE_GROUP,
};
pub use diagnostic::ShaderDispatchBuildDiagnostic;
pub use fullscreen_pass::{
    FullscreenPassBuilder, FullscreenPassPlan, FullscreenPipelineCacheKey, FullscreenShaderRef,
    FULLSCREEN_FIRST_PASS_INPUT_BINDING, FULLSCREEN_FRAME_GROUP, FULLSCREEN_PARAMS_BINDING,
    FULLSCREEN_PASS_INPUT_GROUP, FULLSCREEN_TRIANGLE_VERTEX_ENTRY,
};
pub use parameter_packer::ShaderParameterValue;
pub use resource_bindings::ShaderNamedResourceBinding;

pub use crate::core::framework::render::{
    RenderShaderEntryPointDescriptor, RenderShaderStage, ShaderAssetKind, ShaderResourceAccess,
    ShaderResourceDescriptor, ShaderResourceKind,
};
