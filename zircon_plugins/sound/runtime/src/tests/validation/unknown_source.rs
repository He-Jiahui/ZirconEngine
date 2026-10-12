// 先注册指向不存在声源的绑定，再核对应用阶段返回 UnknownSource。
use super::super::super::*;

#[test]
fn automation_binding_reports_unknown_source_targets() {
    let sound = DefaultSoundManager::default();
    let unknown_source_binding = SoundAutomationBindingId::new(22);

    sound
        .bind_automation(SoundAutomationBinding {
            id: unknown_source_binding,
            timeline_track_path: "Root/Source:sound.source.gain".to_string(),
            target: SoundAutomationTarget::Source(SoundSourceId::new(404)),
            parameter: SoundParameterId::new("gain"),
        })
        .unwrap();

    assert!(matches!(
        sound
            .apply_automation_value(unknown_source_binding, 0.25)
            .unwrap_err(),
        SoundError::UnknownSource { .. }
    ));
}
