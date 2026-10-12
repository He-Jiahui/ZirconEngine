//! 运行时状态机编译、条件、混合空间、层和转换契约的总导出层。
#[path = "blend_space/mod.rs"]
mod blend_space;
#[path = "compiled/mod.rs"]
mod compiled;
#[path = "condition_expression/mod.rs"]
mod condition_expression;
#[path = "layer/mod.rs"]
mod layer;
#[path = "transition/mod.rs"]
mod transition;

pub use blend_space::BlendSpaceCompileError;
pub use compiled::{
    compile_animation_state_machine_runtime, AnimationStateMachineCompileError,
    CompiledAnimationStateMachine, CompiledStateMachineEvaluation,
};
pub(crate) use compiled::{
    compile_animation_state_machine_runtime_bundle, CompiledGraphSamples,
    StateMachineBlendSamplingState,
};
pub use condition_expression::{
    CompiledConditionExpression, ConditionExpression, ConditionExpressionCompileError,
};
pub use layer::{
    compile_animation_state_machine_layers_runtime, CompiledStateMachineLayer,
    CompiledStateMachineLayers, StateMachineLayerCompileError,
};
pub use transition::{
    InterruptionPolicy, TransitionDesc, TransitionRequest, TransitionRuntime, TransitionState,
    TransitionWeights,
};
