//! 自动化绑定入库前规范化轨道路径；时间序列与直接赋值借此使用相同的目标身份。
use zircon_runtime::core::framework::{
    animation::AnimationTrackPath,
    sound::{SoundAutomationBinding, SoundError},
};

pub(crate) fn normalized_automation_binding(
    mut binding: SoundAutomationBinding,
) -> Result<SoundAutomationBinding, SoundError> {
    binding.timeline_track_path = normalize_timeline_track_path(&binding.timeline_track_path)?;
    Ok(binding)
}

fn normalize_timeline_track_path(path: &str) -> Result<String, SoundError> {
    let path = path.trim();
    if path.is_empty() {
        return Err(SoundError::InvalidParameter(
            "automation binding requires a timeline track path".to_string(),
        ));
    }

    AnimationTrackPath::parse(path)
        .map(|path| path.to_string())
        .map_err(|_| {
            SoundError::InvalidParameter(
                "automation binding requires an AnimationTrackPath-style timeline track path"
                    .to_string(),
            )
        })
}
