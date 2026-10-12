//! 状态机编译产物的公共导出层；帧实例缓存依赖该布局与资产修订保持一致。
#[path = "../../animation_state_machine_compile_error.rs"]
mod animation_state_machine_compile_error;
#[path = "compile.rs"]
mod compile;
#[path = "../animation_state_machine.rs"]
mod compiled_animation_state_machine;
#[path = "compiled_state.rs"]
mod compiled_state;
#[path = "evaluation.rs"]
mod compiled_state_machine_evaluation;
#[path = "../compiled_transition.rs"]
mod compiled_transition;
#[path = "evaluate.rs"]
mod evaluate;
#[path = "state_slot.rs"]
mod state_slot;

pub use animation_state_machine_compile_error::AnimationStateMachineCompileError;
pub use compile::compile_animation_state_machine_runtime;
pub(crate) use compile::compile_animation_state_machine_runtime_bundle;
pub use compiled_animation_state_machine::CompiledAnimationStateMachine;
pub use compiled_state_machine_evaluation::CompiledStateMachineEvaluation;

pub(crate) use compiled_state::{CompiledGraphSamples, StateMachineBlendSamplingState};
use compiled_state::{CompiledState, CompiledStateKind};
use compiled_transition::CompiledTransition;
pub(crate) use evaluate::StateMachineParameterValues;
use state_slot::StateSlot;
