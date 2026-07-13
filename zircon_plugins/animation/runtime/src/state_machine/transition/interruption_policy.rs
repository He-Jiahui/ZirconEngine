#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum InterruptionPolicy {
    #[default]
    None,
    CurrentToNext,
    NextToNext,
    Both,
}

impl From<zircon_runtime::core::framework::animation::AnimationTransitionInterruptionPolicyAsset>
    for InterruptionPolicy
{
    fn from(
        value: zircon_runtime::core::framework::animation::AnimationTransitionInterruptionPolicyAsset,
    ) -> Self {
        use zircon_runtime::core::framework::animation::AnimationTransitionInterruptionPolicyAsset as Asset;
        match value {
            Asset::None => Self::None,
            Asset::CurrentToNext => Self::CurrentToNext,
            Asset::NextToNext => Self::NextToNext,
            Asset::Both => Self::Both,
        }
    }
}
