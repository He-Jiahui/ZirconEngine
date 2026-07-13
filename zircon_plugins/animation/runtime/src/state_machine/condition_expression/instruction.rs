use zircon_runtime::core::framework::animation::AnimationConditionOperatorAsset;
use zircon_runtime::core::framework::animation::AnimationParameterValue;

use super::parameter_table::ParameterSlot;

#[derive(Clone, Debug)]
pub(super) enum ConditionInstruction {
    Compare {
        parameter: ParameterSlot,
        operator: AnimationConditionOperatorAsset,
        value: Option<AnimationParameterValue>,
    },
    All(u32),
    Any(u32),
    Not,
}
