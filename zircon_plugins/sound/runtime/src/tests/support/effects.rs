// 为效果行为用例提供固定测试身份；同一条轨道需要多个效果时，调用方须另行分配不同 ID。
use zircon_runtime::core::framework::sound::{
    SoundEffectDescriptor, SoundEffectId, SoundEffectKind,
};

pub(in crate::tests) fn test_effect(kind: SoundEffectKind) -> SoundEffectDescriptor {
    SoundEffectDescriptor::new(SoundEffectId::new(99), "Test Effect", kind)
}
