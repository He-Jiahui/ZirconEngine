//! State-machine semantic validation and canonical IR.

mod compile;
mod model;

pub use compile::compile_animation_state_machine;
pub use model::{
    AnimationCompiledBlendSpace1DSample, AnimationCompiledBlendSpace2DSample,
    AnimationCompiledState, AnimationCompiledStateKind, AnimationCompiledStateMachine,
    AnimationCompiledStateMachineLayer, AnimationCompiledTransition,
    AnimationCompiledTransitionCondition, AnimationStateMachineCompilation,
};

#[cfg(test)]
#[path = "tests/cases.rs"]
mod tests;
