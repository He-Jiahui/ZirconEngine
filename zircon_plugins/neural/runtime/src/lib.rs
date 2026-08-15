mod capability;
mod cpu;
mod gpu;
mod model;
mod ops;
mod plugin;

pub use capability::{
    NATIVE_PLUGIN_ID, NATIVE_REQUESTED_CAPABILITIES, NATIVE_RUNTIME_ENTRY,
    NATIVE_RUNTIME_REGISTRATION_MANIFEST, NEURAL_DECLARATION, NEURAL_MODEL_ASSET_CAPABILITY,
    NEURAL_POST_PROCESS_FEATURE_ID, NEURAL_POST_PROCESS_RUNTIME_CAPABILITY,
    NEURAL_RUNTIME_CAPABILITY, PLUGIN_ID, RENDERING_POST_PROCESS_RUNTIME_CAPABILITY,
    RUNTIME_CAPABILITIES,
};
pub use cpu::{run_cpu, NnCpuError};
pub use gpu::{
    NnGraphBuildError, NnGraphExecutor, NnGraphIo, NnGraphPassPlan, NnTensorLayout,
    NnTensorLayoutError, NnWeightUploadPlan, NnWeightUploadPlanError,
};
pub use model::{
    NnDataType, NnModelAsset, NnModelFormatError, NnModelValidationError, NnTensorDesc,
    NnTensorKind,
};
pub use ops::{NnConv2dAttrs, NnGemmAttrs, NnOp, NnOpAttrs, NnOpCode, NnPool2dAttrs};
pub use plugin::{
    neural_post_process_feature_manifest, package_manifest, plugin_registration,
    runtime_capabilities, runtime_plugin, runtime_plugin_descriptor, runtime_selection,
    NeuralRuntimePlugin, NEURAL_DIST_CRATE_NAME,
};

pub const NN_WEIGHT_ALIGNMENT: u64 = 256;

#[cfg(test)]
mod tests;
