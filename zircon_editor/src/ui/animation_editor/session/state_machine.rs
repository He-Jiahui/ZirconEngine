use zircon_runtime::core::framework::animation::AnimationStateTransitionAsset;

pub(super) fn transition_label(transition: &AnimationStateTransitionAsset) -> String {
    format!("{} -> {}", transition.from_state, transition.to_state)
}
