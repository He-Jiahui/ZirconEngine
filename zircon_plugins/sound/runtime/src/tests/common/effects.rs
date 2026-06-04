use zircon_runtime::core::framework::sound::{
    SoundEffectDescriptor, SoundEffectId, SoundEffectKind,
};

pub(in crate::tests) fn test_effect(kind: SoundEffectKind) -> SoundEffectDescriptor {
    SoundEffectDescriptor::new(SoundEffectId::new(99), "Test Effect", kind)
}
