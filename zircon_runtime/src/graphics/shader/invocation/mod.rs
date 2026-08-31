mod compiler;
mod diagnostic;
mod fullscreen_pass;
mod parameter_packer;
mod resource_bindings;

pub use compiler::{
    COMPUTE_SHADER_FIRST_RESOURCE_BINDING, COMPUTE_SHADER_PARAMS_BINDING,
    COMPUTE_SHADER_RESOURCE_GROUP, ComputeDispatchBuilder, ComputeDispatchPlan, ComputeKernelRef,
    ComputePipelineCacheKey, ShaderAbiBinding, ShaderDispatchExtent,
};
pub use diagnostic::ShaderDispatchBuildDiagnostic;
pub use fullscreen_pass::{
    FULLSCREEN_FIRST_PASS_INPUT_BINDING, FULLSCREEN_FRAME_GROUP, FULLSCREEN_PARAMS_BINDING,
    FULLSCREEN_PASS_INPUT_GROUP, FULLSCREEN_TRIANGLE_VERTEX_ENTRY, FullscreenPassBuilder,
    FullscreenPassPlan, FullscreenPipelineCacheKey, FullscreenShaderRef,
};
pub use parameter_packer::ShaderParameterValue;
pub use resource_bindings::ShaderNamedResourceBinding;

pub(crate) use compiler::{validate_named_resource_bindings, validate_shader_entry_point};

pub use crate::core::framework::render::{
    RenderShaderEntryPointDescriptor, RenderShaderStage, ShaderAssetKind, ShaderResourceAccess,
    ShaderResourceDescriptor, ShaderResourceKind,
};
