use super::super::super::*;

use super::support::{
    assert_next_execution_skipped_missing_executor, register_cleanup_event_and_handler,
    register_cleanup_event_handler_and_executor, submit_cleanup_invocation, CleanupFixture,
};

#[test]
fn configure_mixer_removes_executors_for_removed_dynamic_events() {
    let sound = DefaultSoundManager::default();
    let fixture = CleanupFixture::graph_reconfigure();

    register_cleanup_event_handler_and_executor(&sound, &fixture);
    sound
        .configure_mixer(SoundMixerGraph::default_stereo(48_000))
        .unwrap();

    register_cleanup_event_and_handler(&sound, &fixture);
    submit_cleanup_invocation(&sound, &fixture);

    assert_next_execution_skipped_missing_executor(&sound);
}
