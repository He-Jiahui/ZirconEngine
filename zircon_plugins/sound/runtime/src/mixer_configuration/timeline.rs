//! 整体换图后删除引用已撤销绑定的时间线，保留剩余序列的进度；局部解绑入口需另行保证推进时的失败语义。
use crate::engine::SoundEngineState;

pub(crate) fn retain_timeline_sequences_for_automation_bindings(state: &mut SoundEngineState) {
    let SoundEngineState {
        automation_bindings,
        timeline_sequences,
        ..
    } = state;
    timeline_sequences.retain(|playback| {
        playback
            .sequence
            .tracks
            .iter()
            .all(|track| automation_bindings.contains_key(&track.binding))
    });
}

#[cfg(test)]
#[path = "tests/timeline.rs"]
mod tests;
