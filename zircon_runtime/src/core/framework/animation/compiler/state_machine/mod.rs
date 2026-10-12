//! State-machine semantic validation and canonical IR.

#[path = "compile.rs"]
mod compile;
#[path = "model.rs"]
mod model;

pub use compile::compile_animation_state_machine;
pub use model::{
    AnimationCompiledBlendSpace1DSample, AnimationCompiledBlendSpace2DSample,
    AnimationCompiledState, AnimationCompiledStateKind, AnimationCompiledStateMachine,
    AnimationCompiledStateMachineLayer, AnimationCompiledTransition,
    AnimationCompiledTransitionCondition, AnimationStateMachineCompilation,
};

#[cfg(test)]
#[path = "../../../../animation/compiler/tests/state_machine_cases.rs"]
mod tests;
