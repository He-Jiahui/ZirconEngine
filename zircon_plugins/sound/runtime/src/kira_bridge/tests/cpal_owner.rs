use kira::{
    backend::mock::MockBackend,
    sound::static_sound::{StaticSoundData, StaticSoundSettings},
    AudioManagerSettings, Frame,
};

use super::{CallbackGate, ProviderRetirement};
use crate::kira_bridge::KiraEngine;
use std::sync::{mpsc::sync_channel, Arc};
use std::time::{Duration, Instant};
use zircon_runtime::core::framework::sound::{SoundPlaybackId, SoundTrackId};

#[test]
fn callback_gate_timeout_resumes_then_retries_to_idle() {
    let gate = Arc::new(CallbackGate::new());
    let lease = gate.enter().expect("gate starts admitting callbacks");
    assert!(!gate.retire_until(Instant::now() + Duration::from_millis(1)));
    drop(lease);
    assert!(gate.retire_until(Instant::now() + Duration::from_millis(10)));
}

struct BlockingProvider {
    drop_started: std::sync::mpsc::SyncSender<()>,
    release: std::sync::mpsc::Receiver<()>,
}

impl Drop for BlockingProvider {
    fn drop(&mut self) {
        let _ = self.drop_started.send(());
        let _ = self.release.recv();
    }
}

#[test]
fn provider_timeout_retains_worker_for_ack_census_retry() {
    let (drop_started_tx, drop_started_rx) = sync_channel(0);
    let (release_tx, release_rx) = sync_channel(0);
    let mut retirement = ProviderRetirement::start(BlockingProvider {
        drop_started: drop_started_tx,
        release: release_rx,
    })
    .expect("provider retirement worker must start");

    let first = retirement
        .finish_until(Instant::now() + Duration::from_millis(1))
        .expect_err("blocked provider must remain pending at the first deadline");
    assert_eq!(first.census().expected_worker_count, 1);
    assert_eq!(first.census().provider_drop_ack_count, 0);

    drop_started_rx
        .recv_timeout(Duration::from_secs(1))
        .expect("retirement worker must reach the real provider Drop before ACK");
    assert_eq!(first.census().provider_drop_ack_count, 0);
    release_tx.send(()).unwrap();
    let complete = retirement
        .finish_until(Instant::now() + Duration::from_secs(1))
        .expect("the retained provider worker must be retryable");
    assert_eq!(complete.provider_drop_ack_count, 1);
    assert_eq!(complete.exited_worker_count, 1);
    assert_eq!(complete.joined_worker_count, 1);
    assert!(complete.complete());
}

fn looping_clip() -> StaticSoundData {
    StaticSoundData {
        sample_rate: 48_000,
        frames: Arc::from([Frame::from_mono(0.2); 4_096]),
        settings: StaticSoundSettings::default().loop_region(..),
        slice: None,
    }
}

#[test]
fn provider_timeout_retains_worker_and_rejects_controls_until_retry() {
    let (drop_started_tx, drop_started_rx) = sync_channel(0);
    let (release_tx, release_rx) = sync_channel(0);
    let mut retirement = ProviderRetirement::start(BlockingProvider {
        drop_started: drop_started_tx,
        release: release_rx,
    })
    .expect("provider retirement worker must start");

    let first = retirement
        .finish_until(Instant::now() + Duration::from_millis(1))
        .expect_err("blocked provider must remain pending at the first deadline");
    assert_eq!(first.census().expected_worker_count, 1);
    assert_eq!(first.census().provider_drop_ack_count, 0);
    drop_started_rx
        .recv_timeout(Duration::from_secs(1))
        .expect("the real provider Drop must begin before an ACK is observable");
    assert_eq!(first.census().provider_drop_ack_count, 0);

    let mut engine = KiraEngine::<MockBackend>::inactive();
    engine
        .activate(AudioManagerSettings::default())
        .expect("mock Kira manager must activate");
    let playback = SoundPlaybackId::new(91);
    engine
        .play(playback, SoundTrackId::master(), looping_clip())
        .expect("the retained generation must own the playback handle");
    engine.mark_provider_retiring_for_test();
    assert!(!engine.is_active());
    assert!(engine.pause(playback).is_err());
    assert!(engine.stop(playback).is_err());
    assert!(engine.contains_playback(playback));

    release_tx.send(()).unwrap();
    let complete = retirement
        .finish_until(Instant::now() + Duration::from_secs(1))
        .expect("the retained provider worker must be retryable");
    assert_eq!(complete.provider_drop_ack_count, 1);
    assert_eq!(complete.exited_worker_count, 1);
    assert_eq!(complete.joined_worker_count, 1);
    assert!(complete.complete());
}
