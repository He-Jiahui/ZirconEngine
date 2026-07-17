use std::sync::{Arc, Mutex};

use kira::{
    backend::{Backend, Renderer},
    sound::{
        static_sound::{StaticSoundData, StaticSoundSettings},
        PlaybackState,
    },
    AudioManagerSettings, Frame,
};
use zircon_runtime::core::framework::sound::{
    SoundMixerGraph, SoundPlaybackId, SoundTrackDescriptor, SoundTrackId, SoundTrackSend,
};

use crate::kira_bridge::KiraEngine;

const TEST_SIGNAL_AMPLITUDE: f32 = 0.25;

#[derive(Clone)]
struct CaptureBackendSettings {
    sample_rate: u32,
    samples: Arc<Mutex<Vec<f32>>>,
}

impl Default for CaptureBackendSettings {
    fn default() -> Self {
        Self {
            sample_rate: 48_000,
            samples: Arc::new(Mutex::new(Vec::new())),
        }
    }
}

struct CaptureBackend {
    renderer: Option<Renderer>,
    samples: Arc<Mutex<Vec<f32>>>,
    buffer: Vec<f32>,
}

impl CaptureBackend {
    fn render(&mut self) {
        let renderer = self.renderer.as_mut().expect("capture backend must start");
        renderer.on_start_processing();
        self.buffer.fill(0.0);
        renderer.process(&mut self.buffer, 2);
        *self
            .samples
            .lock()
            .expect("capture samples must not poison") = self.buffer.clone();
    }
}

impl Backend for CaptureBackend {
    type Settings = CaptureBackendSettings;
    type Error = ();

    fn setup(
        settings: Self::Settings,
        internal_buffer_size: usize,
    ) -> Result<(Self, u32), Self::Error> {
        Ok((
            Self {
                renderer: None,
                samples: settings.samples,
                buffer: vec![0.0; internal_buffer_size * 2],
            },
            settings.sample_rate,
        ))
    }

    fn start(&mut self, renderer: Renderer) -> Result<(), Self::Error> {
        self.renderer = Some(renderer);
        Ok(())
    }
}

#[test]
fn post_effect_send_obeys_target_bus_gain_mute_and_parent_gain() {
    let dry = render_post_effect_send(0.0, 1.0, 1.0, false, 1.0, 1.0);
    let full = render_post_effect_send(1.0, 1.0, 1.0, false, 1.0, 1.0);
    let target_half = render_post_effect_send(1.0, 0.5, 1.0, false, 1.0, 1.0);
    let parent_half = render_post_effect_send(1.0, 1.0, 0.5, false, 1.0, 1.0);
    let muted = render_post_effect_send(1.0, 1.0, 1.0, true, 1.0, 1.0);

    let send_delta = full - dry;
    assert!(
        send_delta > 0.01,
        "post-effect send must contribute audio: dry={dry}, full={full}"
    );
    assert!(
        (target_half - dry - send_delta * 0.5).abs() < 0.01,
        "target gain must scale only the send path: dry={dry}, full={full}, target_half={target_half}"
    );
    assert!(
        (parent_half - dry - send_delta * 0.5).abs() < 0.01,
        "parent gain must scale only the send path: dry={dry}, full={full}, parent_half={parent_half}"
    );
    assert!(
        (muted - dry).abs() < 0.01,
        "muting the target must silence only the send path: dry={dry}, muted={muted}"
    );
}

#[test]
fn global_volume_gain_scales_the_rendered_main_output() {
    let full = render_post_effect_send(0.0, 1.0, 1.0, false, 1.0, 1.0);
    let quarter = render_post_effect_send(0.0, 1.0, 1.0, false, 0.25, 1.0);

    assert!(full > 0.01, "the rendered output must contain audio");
    assert!(
        (quarter - full * 0.25).abs() < 0.01,
        "global gain must scale the final output: full={full}, quarter={quarter}"
    );
}

#[test]
fn master_track_gain_is_applied_once_to_direct_and_send_paths() {
    let full = render_post_effect_send(1.0, 1.0, 1.0, false, 1.0, 1.0);
    let half = render_post_effect_send(1.0, 1.0, 1.0, false, 1.0, 0.5);

    assert!(full > 0.01, "the rendered output must contain audio");
    assert!(
        (half - full * 0.5).abs() < 0.01,
        "master gain must be applied once: full={full}, half={half}"
    );
}

#[test]
fn active_graph_sync_updates_the_rendered_send_for_parent_gain_changes() {
    let captures = Arc::new(Mutex::new(Vec::new()));
    let settings = AudioManagerSettings {
        backend_settings: CaptureBackendSettings {
            sample_rate: 48_000,
            samples: Arc::clone(&captures),
        },
        ..AudioManagerSettings::default()
    };
    let mut engine = KiraEngine::<CaptureBackend>::inactive();
    engine.activate(settings).unwrap();
    let graph = send_graph(1.0, 1.0, false, 1.0);
    engine.sync_graph(&graph).unwrap();
    engine
        .play(
            SoundPlaybackId::new(1),
            SoundTrackId::new(2),
            looping_constant_clip(),
        )
        .unwrap();
    let full = render_stable_peak(&mut engine, &captures);

    let mut updated = graph;
    updated.tracks[2].controls.gain = 0.25;
    engine.sync_graph(&updated).unwrap();
    let quarter_send = render_stable_peak(&mut engine, &captures);
    let dry = render_post_effect_send(0.0, 1.0, 1.0, false, 1.0, 1.0);

    assert!(
        full - dry > 0.01,
        "the active send must contribute audio: dry={dry}, full={full}"
    );
    assert!(
        (quarter_send - dry - (full - dry) * 0.25).abs() < 0.01,
        "graph sync must update the active send gain: dry={dry}, full={full}, quarter_send={quarter_send}"
    );
}

#[test]
fn active_send_gain_update_keeps_playback_alive_and_updates_the_existing_route() {
    let captures = Arc::new(Mutex::new(Vec::new()));
    let mut engine = capture_engine(&captures);
    let graph = send_graph(1.0, 1.0, false, 0.25);
    engine.sync_graph(&graph).unwrap();
    engine
        .play(
            SoundPlaybackId::new(1),
            SoundTrackId::new(2),
            looping_constant_clip(),
        )
        .unwrap();
    let quarter_send = render_stable_peak(&mut engine, &captures);

    let mut updated = graph;
    updated.tracks[1].sends[0].gain = 1.0;
    engine.sync_graph(&updated).unwrap();
    let full_send = render_stable_peak(&mut engine, &captures);

    assert!(engine.contains_playback(SoundPlaybackId::new(1)));
    assert!(
        full_send > quarter_send + 0.1,
        "send gain must update without dropping active playback: quarter={quarter_send}, full={full_send}"
    );
}

#[test]
fn stopped_playback_handle_does_not_block_structural_graph_sync_before_drain() {
    let captures = Arc::new(Mutex::new(Vec::new()));
    let mut engine = capture_engine(&captures);
    let graph = send_graph(1.0, 1.0, false, 0.0);
    engine.sync_graph(&graph).unwrap();
    let playback = SoundPlaybackId::new(1);
    engine
        .play(playback, SoundTrackId::new(2), constant_clip())
        .unwrap();

    for _ in 0..128 {
        engine
            .with_backend_mut(CaptureBackend::render)
            .expect("capture backend must be active");
        if engine.playback_state(playback).unwrap() == PlaybackState::Stopped {
            break;
        }
    }
    assert_eq!(
        engine.playback_state(playback).unwrap(),
        PlaybackState::Stopped
    );
    assert!(engine.contains_playback(playback));

    let mut updated = graph;
    updated.tracks[1].parent = Some(SoundTrackId::new(4));
    engine.sync_graph(&updated).unwrap();

    assert_eq!(engine.installed_graph_for_test(), Some(&updated));
    assert!(engine.contains_playback(playback));
    assert_eq!(engine.drain_finished_playbacks(), vec![playback]);
}

#[test]
fn active_parent_move_is_rejected_without_retiring_the_live_playback_track() {
    let captures = Arc::new(Mutex::new(Vec::new()));
    let mut engine = capture_engine(&captures);
    let graph = send_graph(1.0, 1.0, false, 0.0);
    engine.sync_graph(&graph).unwrap();
    engine
        .play(
            SoundPlaybackId::new(1),
            SoundTrackId::new(2),
            looping_constant_clip(),
        )
        .unwrap();
    let before = render_stable_peak(&mut engine, &captures);

    let mut updated = graph;
    updated.tracks[1].parent = Some(SoundTrackId::new(4));
    let error = engine.sync_graph(&updated).unwrap_err();
    let after = render_stable_peak(&mut engine, &captures);

    assert!(matches!(
        error,
        zircon_runtime::core::framework::sound::SoundError::UnsupportedAdvancedFeature(_)
    ));
    assert_eq!(engine.installed_graph_for_test(), Some(&graph));
    assert!(engine.contains_playback(SoundPlaybackId::new(1)));
    assert!(
        before > 0.01 && after > 0.01,
        "rejected structural edits must leave active playback on the installed graph: before={before}, after={after}"
    );
}

#[test]
fn chained_post_effect_send_contributes_to_every_downstream_target() {
    let captures = Arc::new(Mutex::new(Vec::new()));
    let mut without_chain = send_graph(1.0, 1.0, false, 1.0);
    let downstream = SoundTrackId::new(5);
    without_chain
        .tracks
        .push(SoundTrackDescriptor::child(downstream, "Downstream"));
    let baseline = render_graph_peak(without_chain.clone(), &captures);

    let aux = SoundTrackId::new(3);
    without_chain
        .tracks
        .iter_mut()
        .find(|track| track.id == aux)
        .unwrap()
        .sends
        .push(SoundTrackSend {
            target: downstream,
            gain: 0.5,
            pre_effects: false,
        });
    let chained = render_graph_peak(without_chain, &captures);

    assert!(
        (chained - baseline - TEST_SIGNAL_AMPLITUDE * 0.5).abs() < 0.01,
        "A -> B -> C must retain B's downstream send: baseline={baseline}, chained={chained}"
    );
}

fn render_post_effect_send(
    send_gain: f32,
    target_gain: f32,
    parent_gain: f32,
    target_muted: bool,
    global_gain: f32,
    master_gain: f32,
) -> f32 {
    let captures = Arc::new(Mutex::new(Vec::new()));
    let mut engine = capture_engine(&captures);
    let mut graph = send_graph(target_gain, parent_gain, target_muted, send_gain);
    graph.tracks[0].controls.gain = master_gain;
    engine.sync_graph(&graph).unwrap();
    engine.set_global_volume(global_gain).unwrap();
    engine
        .play(
            SoundPlaybackId::new(1),
            SoundTrackId::new(2),
            constant_clip(),
        )
        .unwrap();
    render_stable_peak(&mut engine, &captures)
}

fn render_graph_peak(graph: SoundMixerGraph, captures: &Arc<Mutex<Vec<f32>>>) -> f32 {
    let mut engine = capture_engine(captures);
    engine.sync_graph(&graph).unwrap();
    engine
        .play(
            SoundPlaybackId::new(1),
            SoundTrackId::new(2),
            constant_clip(),
        )
        .unwrap();
    render_stable_peak(&mut engine, captures)
}

fn capture_engine(captures: &Arc<Mutex<Vec<f32>>>) -> KiraEngine<CaptureBackend> {
    let settings = AudioManagerSettings {
        backend_settings: CaptureBackendSettings {
            sample_rate: 48_000,
            samples: Arc::clone(captures),
        },
        ..AudioManagerSettings::default()
    };
    let mut engine = KiraEngine::<CaptureBackend>::inactive();
    engine.activate(settings).unwrap();
    engine
}

fn constant_clip() -> StaticSoundData {
    StaticSoundData {
        sample_rate: 48_000,
        // Leave headroom so Kira's final [-1.0, 1.0] output clamp cannot hide
        // the additional send path from these routing assertions.
        frames: Arc::from([Frame::from_mono(TEST_SIGNAL_AMPLITUDE); 4_096]),
        settings: StaticSoundSettings::default(),
        slice: None,
    }
}

fn looping_constant_clip() -> StaticSoundData {
    let mut clip = constant_clip();
    clip.settings = clip.settings.loop_region(..);
    clip
}

fn render_stable_peak(
    engine: &mut KiraEngine<CaptureBackend>,
    captures: &Arc<Mutex<Vec<f32>>>,
) -> f32 {
    for _ in 0..8 {
        engine
            .with_backend_mut(CaptureBackend::render)
            .expect("capture backend must be active");
    }
    captures
        .lock()
        .expect("capture samples must not poison")
        .iter()
        .step_by(2)
        .copied()
        .fold(0.0_f32, f32::max)
}

fn send_graph(
    target_gain: f32,
    parent_gain: f32,
    target_muted: bool,
    send_gain: f32,
) -> SoundMixerGraph {
    let mut graph = SoundMixerGraph::default_stereo(48_000);
    let music = SoundTrackId::new(2);
    let aux = SoundTrackId::new(3);
    let bus = SoundTrackId::new(4);
    let mut music_track = SoundTrackDescriptor::child(music, "Music");
    music_track.sends.push(SoundTrackSend {
        target: aux,
        gain: send_gain,
        pre_effects: false,
    });
    let mut aux_track = SoundTrackDescriptor::child(aux, "Aux");
    aux_track.parent = Some(bus);
    aux_track.controls.gain = target_gain;
    aux_track.controls.mute = target_muted;
    let mut bus_track = SoundTrackDescriptor::child(bus, "Bus");
    bus_track.controls.gain = parent_gain;

    graph.tracks.extend([music_track, bus_track, aux_track]);
    graph
}
