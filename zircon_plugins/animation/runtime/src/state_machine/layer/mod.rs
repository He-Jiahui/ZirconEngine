//! 层编译和产物导出边界；掩码权重需与目标骨架行序相配。
mod compile;
mod compile_error;
mod compiled_layer;
mod compiled_layers;

pub use compile::compile_animation_state_machine_layers_runtime;
pub use compile_error::StateMachineLayerCompileError;
pub use compiled_layer::CompiledStateMachineLayer;
pub use compiled_layers::CompiledStateMachineLayers;
