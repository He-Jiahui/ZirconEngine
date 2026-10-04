use std::sync::{mpsc::sync_channel, Arc};
use std::time::Duration;

use zircon_runtime::asset::{AssetUri, SoundAsset};
use zircon_runtime::core::framework::audio::AudioChannelLayout;
use zircon_runtime::core::framework::sound::{
    SoundClipId, SoundOutputDeviceManager, SoundPlaybackCompletionAction, SoundSourceDescriptor,
    SoundSourceManager, SoundTrackId,
};

use super::*;

#[test]
fn generation_fence_rejects_stop_or_config_changes_before_commit() {
    let config = SoundConfig::default();
    let mut state = SoundEngineState::new(&config);
    let descriptor = state.output_device.descriptor().clone();
    let snapshot = output_generation_snapshot(&state, &config, descriptor);

    state.output_generation = state.output_generation.wrapping_add(1);
    assert!(!transition_snapshot_matches(&state, &snapshot));

    state.output_generation = snapshot.output_generation;
    state.graph_revision = state.graph_revision.wrapping_add(1);
    assert!(!transition_snapshot_matches(&state, &snapshot));
}

#[test]
fn owner_gain_writer_waits_for_actual_config_admission() {
    let _serial = owner_test_serial();
    let manager = Arc::new(DefaultSoundManager::default());
    SoundOutputDeviceManager::start_output_device(&*manager)
        .expect("managed native CPAL gate must provide the selected output device");
    let descriptor_b = {
        let state = lock_recover(&manager.state);
        let mut descriptor = state.output_device.descriptor().clone();
        descriptor.display_name.push_str("/owner-admission-B");
        descriptor
    };
    let (owner_entered_tx, owner_entered_rx) = sync_channel(0);
    let (owner_release_tx, owner_release_rx) = sync_channel(0);
    install_owner_config_admission_barrier(owner_entered_tx, owner_release_rx);
    let (writer_attempted_tx, writer_attempted_rx) = sync_channel(0);
    let (writer_proceed_tx, writer_proceed_rx) = sync_channel(0);
    let (writer_acquired_tx, writer_acquired_rx) = sync_channel(0);
    install_owner_gain_writer_admission_barrier(
        writer_attempted_tx,
        writer_proceed_rx,
        writer_acquired_tx,
    );

    let owner = Arc::clone(&manager);
    let configure = std::thread::spawn(move || owner.configure_output_device_impl(descriptor_b));
    owner_entered_rx
        .recv_timeout(Duration::from_secs(1))
        .expect("actual owner must hold config while admitting a transition");

    let writer = Arc::clone(&manager);
    let setter = std::thread::spawn(move || writer.set_global_volume_gain_impl(0.25));
    writer_attempted_rx
        .recv_timeout(Duration::from_secs(1))
        .expect("gain setter must reach its owner admission attempt");
    writer_proceed_tx.send(()).unwrap();
    assert!(
        writer_acquired_rx
            .recv_timeout(Duration::from_millis(25))
            .is_err(),
        "setter must remain blocked on the held owner config guard"
    );
    owner_release_tx.send(()).unwrap();
    configure.join().unwrap().unwrap();
    writer_acquired_rx
        .recv_timeout(Duration::from_secs(1))
        .expect("setter must acquire config after owner releases it");
    setter.join().unwrap().unwrap();
    assert_eq!(manager.config().master_gain, 0.25);
    SoundOutputDeviceManager::stop_output_device(&*manager).unwrap();
}

#[test]
fn owner_configure_stop_configure_advances_the_real_generation_fence() {
    let _serial = owner_test_serial();
    let manager = DefaultSoundManager::default();
    manager.set_global_volume_gain_impl(0.4).unwrap();
    // Exercise the production start path as part of the same owner scenario. A missing CPAL
    // device must fail this behavioral test, keeping the native gate explicit rather than
    // converting unavailable hardware into a skipped pass.
    SoundOutputDeviceManager::start_output_device(&manager)
        .expect("managed native CPAL gate must provide the selected output device");
    let clip = manager.insert_clip_for_test(SoundAsset {
        uri: AssetUri::parse("res://sound/owner-aba.wav").unwrap(),
        sample_rate_hz: 48_000,
        channel_count: 1,
        channel_layout: AudioChannelLayout::mono(),
        samples: vec![0.2; 4096],
    });
    let source_id = SoundSourceManager::create_source(&manager, SoundSourceDescriptor::clip(clip))
        .expect("normal owner source creation must bind the active Kira generation");
    let (descriptor_a, source_identity, track_ids, initial_generation, initial_playback) = {
        let state = lock_recover(&manager.state);
        (
            state.output_device.descriptor().clone(),
            state.sources[&source_id].descriptor.clone(),
            state
                .graph
                .tracks
                .iter()
                .map(|track| track.id)
                .collect::<Vec<_>>(),
            state.output_generation,
            state.sources[&source_id].kira_playback,
        )
    };
    assert!(initial_playback.is_some());
    let descriptor_b = manager
        .available_output_devices_impl()
        .expect("normal owner catalog must enumerate managed CPAL devices")
        .into_iter()
        .filter(|info| info.available && info.descriptor.id != descriptor_a.id)
        .map(|info| info.descriptor)
        .next()
        .expect("managed native CPAL gate must provide a distinct B device identity");

    // A failed B request must exercise the normal CPAL admission path and retain the active
    // A generation before the successful B replacement is attempted.
    let mut failed_b = descriptor_b.clone();
    failed_b.sample_rate_hz = u32::MAX;
    let failed = SoundOutputDeviceManager::configure_output_device(&manager, failed_b);
    assert!(
        failed.is_err(),
        "unsupported B format must fail CPAL admission"
    );
    let retained_after_failed_b = lock_recover(&manager.state);
    assert_eq!(
        retained_after_failed_b.output_device.descriptor(),
        &descriptor_a
    );
    assert_eq!(
        retained_after_failed_b.output_generation,
        initial_generation
    );
    assert_eq!(
        retained_after_failed_b.sources[&source_id].descriptor,
        source_identity
    );
    assert_eq!(
        retained_after_failed_b.sources[&source_id].kira_playback,
        initial_playback
    );
    drop(retained_after_failed_b);

    SoundOutputDeviceManager::configure_output_device(&manager, descriptor_b.clone())
        .expect("normal owner B admission should use the selected physical device");
    let after_b = lock_recover(&manager.state).output_generation;
    assert_eq!(after_b, initial_generation.wrapping_add(1));
    SoundOutputDeviceManager::start_output_device(&manager)
        .expect("normal owner B restart must rebind the retained source");
    let (b_source_identity, b_playback, b_generation) = {
        let state = lock_recover(&manager.state);
        (
            state.sources[&source_id].descriptor.clone(),
            state.sources[&source_id].kira_playback,
            state.output_generation,
        )
    };
    assert_eq!(b_source_identity, source_identity);
    assert!(b_playback.is_some() && b_playback != initial_playback);
    assert_eq!(b_generation, after_b.wrapping_add(1));

    SoundOutputDeviceManager::configure_output_device(&manager, descriptor_a.clone())
        .expect("normal owner A replacement must retain the source identity");
    let after_a = lock_recover(&manager.state).output_generation;
    assert_eq!(after_a, b_generation.wrapping_add(1));
    SoundOutputDeviceManager::start_output_device(&manager)
        .expect("normal owner A retry must rebind the retained source");
    let state = lock_recover(&manager.state);
    assert_eq!(state.output_generation, after_a.wrapping_add(1));
    assert_eq!(state.output_device.descriptor(), &descriptor_a);
    assert_eq!(state.sources[&source_id].descriptor, source_identity);
    assert!(state.sources[&source_id].kira_playback.is_some());
    assert_eq!(
        state
            .graph
            .tracks
            .iter()
            .map(|track| track.id)
            .collect::<Vec<_>>(),
        track_ids
    );
    drop(state);
    assert_eq!(manager.config().sample_rate_hz, descriptor_a.sample_rate_hz);
    assert_eq!(manager.config().master_gain, 0.4);
    SoundOutputDeviceManager::stop_output_device(&manager).unwrap();
}

#[test]
fn owner_rejected_candidate_preserves_registered_playback_and_generation() {
    let _serial = owner_test_serial();
    let manager = DefaultSoundManager::default();
    manager
        .start_output_device_impl()
        .expect("managed native CPAL gate must provide the selected output device");
    let playback = super::SoundPlaybackId::new(77);
    {
        let mut state = lock_recover(&manager.state);
        state.playbacks.insert(
            playback,
            ActivePlayback {
                clip: SoundClipId::new(5),
                cursor_frame: 0,
                cursor_position: 0.0,
                gain: 1.0,
                speed: 1.0,
                looped: true,
                completion_action: SoundPlaybackCompletionAction::None,
                paused: false,
                muted: false,
                range_start_frame: 0,
                range_end_frame: None,
                output_track: SoundTrackId::master(),
                pan: 0.0,
            },
        );
    }
    let config = manager.config();
    let snapshot = {
        let state = lock_recover(&manager.state);
        output_generation_snapshot(&state, &config, state.output_device.descriptor().clone())
    };
    let before_playbacks = snapshot.playbacks.clone();
    let before_generation = snapshot.output_generation;
    let prepared = PreparedOutputGeneration {
        kira: DefaultKiraEngine::inactive(),
        sources: HashMap::new(),
        next_playback_id: snapshot.next_playback_id,
    };

    // This is the production prepare/commit boundary. The lower reservation seam rejects
    // before `mem::replace`; callback PCM continuity is covered only by the managed native
    // gate and is intentionally unrun here.
    crate::kira_bridge::fail_next_retirement_reservation_for_test();
    let error = manager
        .commit_prepared_output_generation(snapshot, prepared)
        .expect_err("retirement preparation failure must reject the owner commit");
    assert!(error.to_string().contains("retired-playback reservation"));

    let state = lock_recover(&manager.state);
    assert!(state.kira.is_active());
    assert_eq!(state.playbacks, before_playbacks);
    assert_eq!(state.output_generation, before_generation);
}

#[test]
fn owner_native_prepare_commit_retains_last_good_after_repeated_failure() {
    let _serial = owner_test_serial();
    let manager = DefaultSoundManager::default();
    manager
        .start_output_device_impl()
        .expect("managed native CPAL gate must provide the selected output device");
    let clip = manager.insert_clip_for_test(SoundAsset {
        uri: AssetUri::parse("res://sound/owner-retry.wav").unwrap(),
        sample_rate_hz: 48_000,
        channel_count: 1,
        channel_layout: AudioChannelLayout::mono(),
        samples: vec![0.2; 4096],
    });
    let source_id = SoundSourceManager::create_source(&manager, SoundSourceDescriptor::clip(clip))
        .expect("normal owner source must be installed before replacement");
    let config = manager.config();
    let snapshot = {
        let state = lock_recover(&manager.state);
        output_generation_snapshot(&state, &config, state.output_device.descriptor().clone())
    };

    // Kira's normal CPAL callback is the PCM oracle; native execution remains an explicit
    // managed gate. Preparation is muted and all owner state is still the last-good one.
    let prepared = prepare_output_generation(&snapshot)
        .expect("normal owner preparation must complete before commit");
    crate::kira_bridge::fail_next_retirement_reservation_for_test();
    assert!(manager
        .commit_prepared_output_generation(snapshot, prepared)
        .is_err());
    let first_failure = lock_recover(&manager.state);
    assert!(first_failure.kira.is_active());
    assert!(first_failure.sources.contains_key(&source_id));
    let retained_generation = first_failure.output_generation;
    drop(first_failure);

    let retry_snapshot = {
        let config = manager.config();
        let state = lock_recover(&manager.state);
        output_generation_snapshot(&state, &config, state.output_device.descriptor().clone())
    };
    let retry_prepared = prepare_output_generation(&retry_snapshot)
        .expect("retry preparation must use the retained owner generation");
    crate::kira_bridge::fail_next_retirement_reservation_for_test();
    assert!(manager
        .commit_prepared_output_generation(retry_snapshot, retry_prepared)
        .is_err());
    assert_eq!(
        lock_recover(&manager.state).output_generation,
        retained_generation
    );

    let success_snapshot = {
        let config = manager.config();
        let state = lock_recover(&manager.state);
        output_generation_snapshot(&state, &config, state.output_device.descriptor().clone())
    };
    let success_prepared = prepare_output_generation(&success_snapshot)
        .expect("retry success must prepare a muted candidate before publication");
    let (retired_entered_tx, retired_entered_rx) = sync_channel(0);
    let (retired_release_tx, retired_release_rx) = sync_channel(0);
    install_owner_retired_drop_barrier(retired_entered_tx, retired_release_rx);
    let commit_manager = Arc::new(manager);
    let commit_owner = Arc::clone(&commit_manager);
    let commit = std::thread::spawn(move || {
        commit_owner.commit_prepared_output_generation(success_snapshot, success_prepared)
    });
    retired_entered_rx
        .recv_timeout(Duration::from_secs(1))
        .expect("retired generation must reach the post-unlock drop barrier");
    let state = lock_recover(&commit_manager.state);
    assert!(state.kira.is_active());
    assert!(state.sources[&source_id].kira_playback.is_some());
    assert_eq!(state.output_generation, retained_generation.wrapping_add(1));
    drop(state);
    retired_release_tx.send(()).unwrap();
    commit
        .join()
        .unwrap()
        .expect("normal owner retry must publish once reservation succeeds");
    let state = lock_recover(&commit_manager.state);
    drop(state);
    commit_manager.stop_output_device_impl().unwrap();
}

#[test]
fn owner_native_callback_quiescence_is_observable_before_stop_returns() {
    let _serial = owner_test_serial();
    crate::kira_bridge::OwnerCpalBackend::reset_callback_telemetry();
    let (entered_tx, entered_rx) = sync_channel(0);
    crate::kira_bridge::OwnerCpalBackend::install_callback_enter_signal(entered_tx);
    let manager = DefaultSoundManager::default();
    manager
        .start_output_device_impl()
        .expect("managed native CPAL gate must provide the selected output device");
    entered_rx
        .recv_timeout(Duration::from_secs(1))
        .expect("normal owner callback must enter the engine-owned generation gate");

    let (retired_tx, retired_rx) = sync_channel(0);
    crate::kira_bridge::OwnerCpalBackend::install_callback_retired_signal(retired_tx);
    manager
        .stop_output_device_impl()
        .expect("normal owner stop must retire the callback generation");
    retired_rx
        .recv_timeout(Duration::from_secs(1))
        .expect("stream drop must follow callback retirement");
    crate::kira_bridge::OwnerCpalBackend::reset_callback_telemetry();
}

#[test]
fn owner_native_prepare_callback_is_silent_until_commit_then_reports_pcm() {
    let _serial = owner_test_serial();
    crate::kira_bridge::OwnerCpalBackend::reset_callback_telemetry();
    let manager = Arc::new(DefaultSoundManager::default());
    let clip = manager.insert_clip_for_test(SoundAsset {
        uri: AssetUri::parse("res://sound/owner-native-callback.wav").unwrap(),
        sample_rate_hz: 48_000,
        channel_count: 1,
        channel_layout: AudioChannelLayout::mono(),
        samples: vec![0.2; 4096],
    });
    let source_id = SoundSourceManager::create_source(&*manager, SoundSourceDescriptor::clip(clip))
        .expect("normal owner source must be retained for prepared binding");
    let (commit_tx, commit_rx) = sync_channel(0);
    let (commit_release_tx, commit_release_rx) = sync_channel(0);
    install_owner_commit_barrier(commit_tx, commit_release_rx);
    let (callback_enter_tx, callback_enter_rx) = sync_channel(1);
    crate::kira_bridge::OwnerCpalBackend::install_callback_enter_signal(callback_enter_tx);
    let (nonzero_tx, nonzero_rx) = sync_channel(1);
    crate::kira_bridge::OwnerCpalBackend::install_callback_nonzero_signal(nonzero_tx);
    let owner = Arc::clone(&manager);
    let start = std::thread::spawn(move || owner.start_output_device_impl());
    commit_rx
        .recv_timeout(Duration::from_secs(1))
        .expect("normal owner must reach the visible commit admission barrier");
    callback_enter_rx
        .recv_timeout(Duration::from_secs(1))
        .expect("prepared normal CPAL callback must run before owner commit");
    assert_eq!(
        crate::kira_bridge::OwnerCpalBackend::callback_peak_for_test(),
        0.0,
        "prepared callback must remain silent before owner commit"
    );
    commit_release_tx.send(()).unwrap();
    start
        .join()
        .unwrap()
        .expect("normal owner commit must complete after the callback boundary");
    nonzero_rx
        .recv_timeout(Duration::from_secs(1))
        .expect("committed owner gain must eventually produce native callback PCM");
    assert!(
        crate::kira_bridge::OwnerCpalBackend::callback_peak_for_test() > 0.01,
        "committed source must produce nonzero callback PCM"
    );
    let state = lock_recover(&manager.state);
    assert!(state.sources[&source_id].kira_playback.is_some());
    drop(state);
    manager.stop_output_device_impl().unwrap();
    crate::kira_bridge::OwnerCpalBackend::reset_callback_telemetry();
}
