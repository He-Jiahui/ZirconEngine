use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use kira::{
    backend::mock::MockBackend,
    backend::{Backend, Renderer},
    sound::static_sound::{StaticSoundData, StaticSoundSettings},
    AudioManagerSettings, Frame,
};
use zircon_runtime::core::framework::sound::{
    SoundClipId, SoundPlaybackId, SoundSourceDescriptor, SoundSourceId, SoundTrackId,
};

use crate::engine::{LoadedClip, SourceVoice};
use crate::kira_bridge::{fail_next_retirement_reservation_for_test, KiraEngine};
use crate::service_types::{prepare_source_generation, set_bound_source_gain};
use crate::tests::test_clip_with_rate;

use super::support::{graph_with_music_track, mock_settings};

#[derive(Clone, Copy)]
enum FailureMode {
    None,
    Setup,
    Start,
}

#[derive(Clone)]
struct FailureBackendSettings {
    mode: FailureMode,
    samples: Arc<Mutex<Vec<f32>>>,
}

impl Default for FailureBackendSettings {
    fn default() -> Self {
        Self {
            mode: FailureMode::None,
            samples: Arc::new(Mutex::new(Vec::new())),
        }
    }
}

struct FailureBackend {
    mode: FailureMode,
    samples: Arc<Mutex<Vec<f32>>>,
    renderer: Option<Renderer>,
    buffer: Vec<f32>,
}

impl FailureBackend {
    fn render(&mut self) {
        let renderer = self.renderer.as_mut().expect("backend must be started");
        renderer.on_start_processing();
        self.buffer.fill(0.0);
        renderer.process(&mut self.buffer, 2);
        *self.samples.lock().expect("capture lock must not poison") = self.buffer.clone();
    }

    fn peak(&self) -> f32 {
        self.samples
            .lock()
            .expect("capture lock must not poison")
            .iter()
            .copied()
            .map(f32::abs)
            .fold(0.0, f32::max)
    }
}

impl Backend for FailureBackend {
    type Settings = FailureBackendSettings;
    type Error = &'static str;

    fn setup(
        settings: Self::Settings,
        internal_buffer_size: usize,
    ) -> Result<(Self, u32), Self::Error> {
        if matches!(settings.mode, FailureMode::Setup) {
            return Err("injected setup failure");
        }
        Ok((
            Self {
                mode: settings.mode,
                samples: settings.samples,
                renderer: None,
                buffer: vec![0.0; internal_buffer_size * 2],
            },
            48_000,
        ))
    }

    fn start(&mut self, renderer: Renderer) -> Result<(), Self::Error> {
        if matches!(self.mode, FailureMode::Start) {
            return Err("injected start failure");
        }
        self.renderer = Some(renderer);
        Ok(())
    }
}

fn settings(
    mode: FailureMode,
    samples: Arc<Mutex<Vec<f32>>>,
) -> AudioManagerSettings<FailureBackend> {
    AudioManagerSettings {
        backend_settings: FailureBackendSettings { mode, samples },
        ..AudioManagerSettings::default()
    }
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
fn setup_and_start_rejection_preserve_the_active_manager_graph_and_voice() {
    let samples = Arc::new(Mutex::new(Vec::new()));
    let mut engine = KiraEngine::<FailureBackend>::inactive();
    engine
        .activate(settings(FailureMode::None, Arc::clone(&samples)))
        .unwrap();
    let graph = graph_with_music_track(48_000);
    engine.sync_graph(&graph).unwrap();
    let playback = SoundPlaybackId::new(7);
    engine
        .play(playback, SoundTrackId::new(2), looping_clip())
        .unwrap();
    for _ in 0..4 {
        engine
            .with_backend_mut(FailureBackend::render)
            .expect("active backend must render");
    }
    let before = engine.with_backend_mut(|backend| backend.peak()).unwrap();
    assert!(before > 0.01, "the original generation must produce PCM");

    assert!(engine
        .activate(settings(FailureMode::Setup, Arc::clone(&samples)))
        .is_err());
    assert!(engine.is_active());
    assert_eq!(engine.installed_graph_for_test(), Some(&graph));
    assert!(engine.contains_playback(playback));
    engine
        .with_backend_mut(FailureBackend::render)
        .expect("setup rejection must leave the old backend active");
    assert!(engine.with_backend_mut(|backend| backend.peak()).unwrap() > 0.01);

    assert!(engine
        .activate(settings(FailureMode::Start, Arc::clone(&samples)))
        .is_err());
    assert!(engine.is_active());
    assert_eq!(engine.installed_graph_for_test(), Some(&graph));
    assert!(engine.contains_playback(playback));
    engine
        .with_backend_mut(FailureBackend::render)
        .expect("start rejection must leave the old backend active");
    assert!(engine.with_backend_mut(|backend| backend.peak()).unwrap() > 0.01);

    // The normal mock backend remains a control for the ordinary consumer closure path.
    let mut closed = KiraEngine::<MockBackend>::inactive();
    closed.activate(mock_settings(48_000)).unwrap();
    let _ = closed.deactivate();
    assert!(!closed.is_active());
}

#[test]
fn source_prepare_failure_stays_before_commit_and_leaves_the_source_registry_unchanged() {
    let mut kira = KiraEngine::<MockBackend>::inactive();
    kira.activate(mock_settings(48_000)).unwrap();
    let source_id = SoundSourceId::new(9);
    let mut sources = HashMap::from([(
        source_id,
        SourceVoice::new(SoundSourceDescriptor::clip(SoundClipId::new(404))),
    )]);
    let before = sources.clone();
    let mut next_playback_id = 10;

    let error = prepare_source_generation(
        &mut kira,
        &mut next_playback_id,
        &HashMap::new(),
        &mut sources,
    )
    .unwrap_err();

    assert!(error.to_string().contains("unknown sound clip"));
    assert_eq!(sources, before);
    assert_eq!(next_playback_id, 10);
    assert!(!kira.contains_playback(SoundPlaybackId::new(11)));
}

#[test]
fn source_prepare_binds_real_clip_with_zero_pcm_until_commit_gain() {
    let clip_id = SoundClipId::new(12);
    let clip = LoadedClip::new(test_clip_with_rate(
        "res://sound/prepared-source.wav",
        48_000,
        &[0.2; 4096],
    ))
    .unwrap();
    let clips = HashMap::from([(clip_id, clip)]);
    let source_id = SoundSourceId::new(13);
    let mut sources = HashMap::from([(
        source_id,
        SourceVoice::new(SoundSourceDescriptor::clip(clip_id)),
    )]);
    let mut next_playback_id = 0;
    let samples = Arc::new(Mutex::new(Vec::new()));
    let mut prepared = KiraEngine::<FailureBackend>::prepare_with_limits(
        settings(FailureMode::None, Arc::clone(&samples)),
        2,
        2,
    )
    .unwrap();

    prepare_source_generation(&mut prepared, &mut next_playback_id, &clips, &mut sources).unwrap();

    assert_eq!(next_playback_id, 1);
    assert_eq!(
        sources[&source_id].kira_playback,
        Some(SoundPlaybackId::new(1))
    );
    prepared.with_backend_mut(FailureBackend::render).unwrap();
    assert_eq!(
        prepared.with_backend_mut(|backend| backend.peak()).unwrap(),
        0.0
    );
    let mut owner = KiraEngine::<FailureBackend>::inactive();
    let prepared_commit = owner.prepare_commit(prepared).unwrap();
    let (_retired, _) = owner.commit_prepared(prepared_commit);
    owner.with_backend_mut(FailureBackend::render).unwrap();
    assert_eq!(
        owner.with_backend_mut(|backend| backend.peak()).unwrap(),
        0.0
    );
    owner.set_global_volume(1.0).unwrap();
    for _ in 0..4 {
        owner.with_backend_mut(FailureBackend::render).unwrap();
    }
    assert!(owner.with_backend_mut(|backend| backend.peak()).unwrap() > 0.01);
}

#[test]
fn prepared_graph_sync_keeps_clip_silent_until_commit_gain() {
    let graph = graph_with_music_track(48_000);
    let clip_id = SoundClipId::new(52);
    let clip = LoadedClip::new(test_clip_with_rate(
        "res://sound/prepared-graph-sync.wav",
        48_000,
        &[0.2; 4096],
    ))
    .unwrap();
    let clips = HashMap::from([(clip_id, clip)]);
    let source_id = SoundSourceId::new(53);
    let mut sources = HashMap::from([(
        source_id,
        SourceVoice::new(SoundSourceDescriptor::clip(clip_id)),
    )]);
    let mut next_playback_id = 0;
    let samples = Arc::new(Mutex::new(Vec::new()));
    let mut prepared = KiraEngine::<FailureBackend>::prepare_with_limits(
        settings(FailureMode::None, Arc::clone(&samples)),
        2,
        2,
    )
    .unwrap();

    // Graph compilation must remain behind the prepared mute before source insertion.
    prepared.set_global_volume(0.0).unwrap();
    prepared.sync_graph(&graph).unwrap();
    prepare_source_generation(&mut prepared, &mut next_playback_id, &clips, &mut sources).unwrap();
    prepared.with_backend_mut(FailureBackend::render).unwrap();
    assert_eq!(
        prepared.with_backend_mut(|backend| backend.peak()).unwrap(),
        0.0
    );

    let mut owner = KiraEngine::<FailureBackend>::inactive();
    let prepared_commit = owner.prepare_commit(prepared).unwrap();
    let (_retired, _) = owner.commit_prepared(prepared_commit);
    owner.with_backend_mut(FailureBackend::render).unwrap();
    assert_eq!(
        owner.with_backend_mut(|backend| backend.peak()).unwrap(),
        0.0
    );
    owner.set_global_volume(1.0).unwrap();
    for _ in 0..4 {
        owner.with_backend_mut(FailureBackend::render).unwrap();
    }
    assert!(owner.with_backend_mut(|backend| backend.peak()).unwrap() > 0.01);
}

#[test]
fn successful_commit_retires_old_playbacks_once_after_prepare() {
    let mut active = KiraEngine::<MockBackend>::inactive();
    active.activate(mock_settings(48_000)).unwrap();
    let graph = graph_with_music_track(48_000);
    active.sync_graph(&graph).unwrap();
    let playback = SoundPlaybackId::new(21);
    active
        .play(playback, SoundTrackId::new(2), looping_clip())
        .unwrap();

    let mut replacement = KiraEngine::<MockBackend>::inactive();
    replacement.activate(mock_settings(48_000)).unwrap();
    replacement.sync_graph(&graph).unwrap();
    let prepared_commit = active.prepare_commit(replacement).unwrap();
    let (_retired, detached) = active.commit_prepared(prepared_commit);

    assert_eq!(detached, vec![playback]);
    assert!(active.is_active());
    assert!(!active.contains_playback(playback));
    assert!(active.deactivate().is_empty());
}

#[test]
fn failed_prepared_graph_does_not_replace_the_renderable_generation() {
    let samples = Arc::new(Mutex::new(Vec::new()));
    let mut active = KiraEngine::<FailureBackend>::inactive();
    active
        .activate(settings(FailureMode::None, Arc::clone(&samples)))
        .unwrap();
    let graph = graph_with_music_track(48_000);
    active.sync_graph(&graph).unwrap();
    let playback = SoundPlaybackId::new(31);
    active
        .play(playback, SoundTrackId::new(2), looping_clip())
        .unwrap();
    for _ in 0..4 {
        active
            .with_backend_mut(FailureBackend::render)
            .expect("active backend must render");
    }
    let before = active.with_backend_mut(|backend| backend.peak()).unwrap();

    let mut prepared = KiraEngine::<FailureBackend>::inactive();
    prepared
        .activate(settings(FailureMode::None, Arc::clone(&samples)))
        .unwrap();
    let mut invalid = graph.clone();
    invalid.tracks[1].parent = Some(SoundTrackId::new(404));
    assert!(prepared.sync_graph(&invalid).is_err());

    assert_eq!(active.installed_graph_for_test(), Some(&graph));
    assert!(active.contains_playback(playback));
    for _ in 0..4 {
        active
            .with_backend_mut(FailureBackend::render)
            .expect("graph rejection must leave the old backend active");
    }
    let after = active.with_backend_mut(|backend| backend.peak()).unwrap();
    assert!(before > 0.01 && after > 0.01);
}

#[test]
fn retirement_reservation_failure_keeps_the_owner_generation_audible() {
    let samples = Arc::new(Mutex::new(Vec::new()));
    let mut active = KiraEngine::<FailureBackend>::inactive();
    active
        .activate(settings(FailureMode::None, Arc::clone(&samples)))
        .unwrap();
    active
        .play(
            SoundPlaybackId::new(44),
            SoundTrackId::new(1),
            looping_clip(),
        )
        .unwrap();
    for _ in 0..4 {
        active.with_backend_mut(FailureBackend::render).unwrap();
    }
    assert!(active.with_backend_mut(|backend| backend.peak()).unwrap() > 0.01);

    let mut replacement = KiraEngine::<FailureBackend>::inactive();
    replacement
        .activate(settings(FailureMode::None, Arc::clone(&samples)))
        .unwrap();
    fail_next_retirement_reservation_for_test();
    let error = match active.prepare_commit(replacement) {
        Ok(_) => panic!("injected retirement reservation must reject the commit"),
        Err(error) => error,
    };
    assert!(error.to_string().contains("retired-playback reservation"));
    assert!(active.is_active());
    assert!(active.contains_playback(SoundPlaybackId::new(44)));
    active.with_backend_mut(FailureBackend::render).unwrap();
    assert!(active.with_backend_mut(|backend| backend.peak()).unwrap() > 0.01);
}

#[test]
fn provider_timeout_retains_manager_handles_but_rejects_normal_consumer_controls() {
    // The real BlockingProvider timeout/census fixture in cpal_owner.rs proves
    // that the provider worker and join authority remain pending after the
    // first deadline. This consumer-side half uses the same retained-generation
    // state: no command, handle removal, or descriptor mutation is permitted
    // until the actual provider retry has completed. No synthetic ACK is used.
    let mut engine = KiraEngine::<MockBackend>::inactive();
    engine.activate(mock_settings(48_000)).unwrap();
    let graph = graph_with_music_track(48_000);
    engine.sync_graph(&graph).unwrap();
    let playback = SoundPlaybackId::new(81);
    engine
        .play(playback, SoundTrackId::new(2), looping_clip())
        .unwrap();
    let mut source = SourceVoice::new(SoundSourceDescriptor::clip(SoundClipId::new(82)));
    source.descriptor.gain = 0.25;

    engine.mark_provider_retiring_for_test();
    assert!(!engine.is_active());
    assert!(engine.pause(playback).is_err());
    assert!(engine.resume(playback).is_err());
    assert!(engine.seek_to(playback, 0.5).is_err());
    assert!(engine.set_volume(playback, 0.5).is_err());
    assert!(engine.set_playback_rate(playback, 1.5).is_err());
    assert!(engine.stop(playback).is_err());
    assert!(engine.set_global_volume(0.5).is_err());
    assert!(engine.contains_playback(playback));

    let before_gain = source.descriptor.gain;
    assert!(set_bound_source_gain(&mut engine, &mut source, 0.75).is_err());
    assert_eq!(source.descriptor.gain, before_gain);
}
