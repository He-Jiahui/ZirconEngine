// 使用手动驱动的 Kira 捕获后端比较混音样本，核对发送路径、总音量和活跃图更新的输出。
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

// 由夹具向 Kira 后端共享最后一块立体声输出，不启动系统音频设备。
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

// 手动推进渲染器的测试后端；测试线程负责调用 render，设备回调时序不在本组覆盖范围内。
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

// 实现真实 Kira 后端契约以保留混音执行；固定双声道缓冲区只适用于本组立体声图。
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

// 活跃播放时改变父轨道须明确拒绝，且拒绝后原图仍可发声；仅核对句柄存在不足以证明音频保留。
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

    let mut updated = graph.clone();
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

// 将发送、目标、父轨道和最终音量分别参数化，供输出对照区分各层增益的作用范围。
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

// 低幅恒定信号为发送路径保留混音余量；若发生最终限幅，增益比例断言就失去识别能力。
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

// 先推进多个缓冲块等待控制渐变，再取左声道正峰值；该指标专用于恒定单声道夹具。
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

// 共用源轨道、辅助轨道和父总线拓扑，故意让辅助轨道继承父总线，以识别发送绕过总线的回归。
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

// Public manager setters prove config/state/cache publication. The same production projection,
// sync and handle controls then drive real Kira PCM without opening a system audio device.
#[test]
fn default_spatial_scale_updates_live_source_gain_and_pan_without_restarting() {
    use crate::service_types::{
        mute_bound_source, pause_bound_source, resume_bound_source, set_bound_source_gain,
        sync_source_voice,
    };
    use zircon_runtime::core::framework::sound::{SoundRuntimeSettingsManager, SoundSourceManager};

    let (manager, source, clips) = spatial_scale_fixture(SoundTrackId::master(), None);
    let mut listener = crate::tests::test_listener();
    manager.update_listener(listener.clone()).unwrap();
    let mut voice = manager
        .source_environment_snapshot_for_test(source)
        .1
        .unwrap();
    assert_eq!((voice.environment_gain, voice.environment_pan), (0.25, 1.0));
    assert!(voice.kira_playback.is_none());
    let captures = Arc::new(Mutex::new(Vec::new()));
    let mut engine = capture_engine(&captures);
    engine
        .sync_graph(&SoundMixerGraph::default_stereo(48_000))
        .unwrap();
    let mut next = 0;
    sync_source_voice(&mut engine, &mut next, &clips, &mut voice).unwrap();
    let playback = voice.kira_playback.unwrap();
    let right = render_stable_stereo(&mut engine, &captures);
    assert!(
        right[1] > 0.02 && right[0] < 0.000_001,
        "right source: {right:?}"
    );
    let position = engine.playback_position(playback).unwrap();
    let cursor = (voice.cursor_frame, voice.cursor_position);

    manager.set_default_spatial_scale(2.0).unwrap();
    refresh_captured_environment(
        &manager,
        source,
        &[listener.clone()],
        &mut voice,
        &mut engine,
    );
    assert_eq!(engine.playback_position(playback).unwrap(), position);
    assert_eq!((voice.cursor_frame, voice.cursor_position), cursor);
    let farther = render_stable_stereo(&mut engine, &captures);
    assert!(
        (farther[1] - right[1] * 0.5).abs() < 0.003 && farther[0] < 0.000_001,
        "distance: {right:?} -> {farther:?}"
    );
    assert!(engine.playback_position(playback).unwrap() > position);

    manager.set_default_spatial_scale(0.0).unwrap();
    refresh_captured_environment(
        &manager,
        source,
        &[listener.clone()],
        &mut voice,
        &mut engine,
    );
    let center = render_stable_stereo(&mut engine, &captures);
    assert!(
        (center[0] - center[1]).abs() < 0.000_001 && center[0] > right[1],
        "zero scale: {center:?}"
    );
    manager.set_source_gain(source, 0.25).unwrap();
    set_bound_source_gain(&mut engine, &mut voice, 0.25).unwrap();
    let quarter = render_stable_stereo(&mut engine, &captures);
    assert!((quarter[0] - center[0] * 0.25).abs() < 0.003);
    manager.mute_source(source).unwrap();
    mute_bound_source(&mut engine, &mut voice, true).unwrap();
    manager.set_default_spatial_scale(1.0).unwrap();
    refresh_captured_environment(
        &manager,
        source,
        &[listener.clone()],
        &mut voice,
        &mut engine,
    );
    let silent = render_stable_stereo(&mut engine, &captures);
    assert!(silent.iter().all(|sample| *sample < 0.000_001));
    manager.set_source_gain(source, 0.5).unwrap();
    set_bound_source_gain(&mut engine, &mut voice, 0.5).unwrap();
    manager.unmute_source(source).unwrap();
    mute_bound_source(&mut engine, &mut voice, false).unwrap();
    let half = render_stable_stereo(&mut engine, &captures);
    assert!(
        (half[1] - right[1] * 0.5).abs() < 0.003 && half[0] < 0.000_001,
        "user gain must retain distance: {half:?}"
    );

    manager.pause_source(source).unwrap();
    pause_bound_source(&mut engine, &mut voice).unwrap();
    render_stable_stereo(&mut engine, &captures);
    assert_eq!(
        engine.playback_state(playback).unwrap(),
        PlaybackState::Paused
    );
    let paused_position = engine.playback_position(playback).unwrap();
    manager.set_default_spatial_scale(2.0).unwrap();
    refresh_captured_environment(
        &manager,
        source,
        &[listener.clone()],
        &mut voice,
        &mut engine,
    );
    render_stable_stereo(&mut engine, &captures);
    assert_eq!(
        engine.playback_state(playback).unwrap(),
        PlaybackState::Paused
    );
    assert_eq!(engine.playback_position(playback).unwrap(), paused_position);
    manager.resume_source(source).unwrap();
    resume_bound_source(&mut engine, &mut voice).unwrap();
    let resumed = render_stable_stereo(&mut engine, &captures);
    assert!(resumed[1] > 0.005 && resumed[0] < 0.000_001);

    listener.position = [8.0, 0.0, 0.0];
    manager.update_listener(listener.clone()).unwrap();
    refresh_captured_environment(
        &manager,
        source,
        &[listener.clone()],
        &mut voice,
        &mut engine,
    );
    let left = render_stable_stereo(&mut engine, &captures);
    assert!(
        left[0] > 0.005 && left[1] < 0.000_001,
        "listener move: {left:?}"
    );
    manager.remove_listener(listener.id).unwrap();
    refresh_captured_environment(&manager, source, &[], &mut voice, &mut engine);
    let no_listener = render_stable_stereo(&mut engine, &captures);
    assert_eq!((voice.environment_gain, voice.environment_pan), (1.0, 0.0));
    assert!((no_listener[0] - no_listener[1]).abs() < 0.000_001 && no_listener[0] > left[0]);
    assert_eq!(voice.kira_playback, Some(playback));
    assert_eq!(
        next, 1,
        "spatial controls must not allocate another playback"
    );
}

#[test]
fn default_spatial_scale_preserves_overrides_and_inactive_source_projection() {
    use crate::service_types::{prepare_source_generation, stop_bound_source, sync_source_voice};
    use zircon_runtime::core::framework::sound::{
        SoundListenerId, SoundMixerGraphManager, SoundRuntimeSettingsManager, SoundSourceManager,
    };

    let track = SoundTrackId::new(2);
    let (manager, source, clips) = spatial_scale_fixture(track, None);
    let master = crate::tests::test_listener();
    let mut other = master.clone();
    other.id = SoundListenerId::new(2);
    other.mixer_target = SoundTrackId::new(3);
    let mut higher_id = master.clone();
    higher_id.id = SoundListenerId::new(9);
    higher_id.mixer_target = track;
    let mut selected = higher_id.clone();
    selected.id = SoundListenerId::new(4);
    selected.position = [8.0, 0.0, 0.0];
    let listeners = vec![
        higher_id.clone(),
        other.clone(),
        selected.clone(),
        master.clone(),
    ];
    for listener in &listeners {
        manager.update_listener(listener.clone()).unwrap();
    }
    manager.set_default_spatial_scale(2.0).unwrap();
    let mut imported_graph = send_graph(1.0, 1.0, false, 0.0);
    imported_graph.sources.push(
        manager
            .source_environment_snapshot_for_test(source)
            .1
            .unwrap()
            .descriptor,
    );
    manager.configure_mixer(imported_graph.clone()).unwrap();
    let mut imported_voice = manager
        .source_environment_snapshot_for_test(source)
        .1
        .unwrap();
    assert_eq!(
        (
            imported_voice.environment_gain,
            imported_voice.environment_pan
        ),
        (0.125, -1.0)
    );
    assert!(imported_voice.kira_playback.is_none());
    // The real inactive graph import must prepare the cache consumed by activation.
    let imported_captures = Arc::new(Mutex::new(Vec::new()));
    let mut imported_engine = capture_engine(&imported_captures);
    imported_engine.sync_graph(&imported_graph).unwrap();
    let mut imported_next = 0;
    sync_source_voice(
        &mut imported_engine,
        &mut imported_next,
        &clips,
        &mut imported_voice,
    )
    .unwrap();
    let imported_pcm = render_stable_stereo(&mut imported_engine, &imported_captures);
    assert!(
        imported_pcm[0] > 0.01 && imported_pcm[1] < 0.000_001,
        "imported default scale: {imported_pcm:?}"
    );
    stop_bound_source(&mut imported_engine, &mut imported_voice).unwrap();
    let mut overridden = imported_voice.descriptor;
    overridden.spatial.spatial_scale = Some(0.5);
    manager.update_source(overridden).unwrap();
    manager.pause_source(source).unwrap();
    manager.set_default_spatial_scale(2.0).unwrap();
    let (_, paused) = manager.source_environment_snapshot_for_test(source);
    let paused = paused.unwrap();
    assert_eq!(
        (paused.environment_gain, paused.environment_pan),
        (0.5, -1.0)
    );
    assert!(!paused.descriptor.playing && paused.kira_playback.is_none());
    manager.set_default_spatial_scale(7.0).unwrap();
    assert_eq!(
        manager
            .source_environment_snapshot_for_test(source)
            .1
            .unwrap(),
        paused
    );
    manager.resume_source(source).unwrap();
    let mut voice = manager
        .source_environment_snapshot_for_test(source)
        .1
        .unwrap();
    let mut engine = KiraEngine::<CaptureBackend>::inactive();
    let mut next = 0;
    sync_source_voice(&mut engine, &mut next, &clips, &mut voice).unwrap();
    assert!(voice.kira_playback.is_none());
    assert_eq!((voice.environment_gain, voice.environment_pan), (0.5, -1.0));

    let captures = Arc::new(Mutex::new(Vec::new()));
    engine
        .activate(AudioManagerSettings {
            backend_settings: CaptureBackendSettings {
                sample_rate: 48_000,
                samples: Arc::clone(&captures),
            },
            ..AudioManagerSettings::default()
        })
        .unwrap();
    engine
        .sync_graph(&send_graph(1.0, 1.0, false, 0.0))
        .unwrap();
    sync_source_voice(&mut engine, &mut next, &clips, &mut voice).unwrap();
    let original_playback = voice.kira_playback.unwrap();
    let left = render_stable_stereo(&mut engine, &captures);
    assert!(
        left[0] > 0.02 && left[1] < 0.000_001,
        "same-track lowest-ID listener: {left:?}"
    );
    stop_bound_source(&mut engine, &mut voice).unwrap();
    assert_eq!((voice.environment_gain, voice.environment_pan), (0.5, -1.0));
    let mut replacement = capture_engine(&captures);
    replacement
        .sync_graph(&send_graph(1.0, 1.0, false, 0.0))
        .unwrap();
    let mut sources = std::collections::HashMap::from([(source, voice)]);
    prepare_source_generation(&mut replacement, &mut next, &clips, &mut sources).unwrap();
    let mut voice = sources.remove(&source).unwrap();
    assert_ne!(voice.kira_playback, Some(original_playback));
    let rebound = render_stable_stereo(&mut replacement, &captures);
    assert!((rebound[0] - left[0]).abs() < 0.003 && rebound[1] < 0.000_001);

    manager.remove_listener(selected.id).unwrap();
    manager.remove_listener(higher_id.id).unwrap();
    refresh_captured_environment(
        &manager,
        source,
        &[other.clone(), master.clone()],
        &mut voice,
        &mut replacement,
    );
    assert_eq!(
        voice.environment_pan, 1.0,
        "master must precede an unrelated track listener"
    );
    manager.remove_listener(master.id).unwrap();
    refresh_captured_environment(
        &manager,
        source,
        &[other.clone()],
        &mut voice,
        &mut replacement,
    );
    assert_eq!(voice.environment_pan, 1.0);
    other.active = false;
    manager.update_listener(other.clone()).unwrap();
    refresh_captured_environment(&manager, source, &[other], &mut voice, &mut replacement);
    assert_eq!((voice.environment_gain, voice.environment_pan), (1.0, 0.0));
    let neutral = render_stable_stereo(&mut replacement, &captures);
    assert!((neutral[0] - neutral[1]).abs() < 0.000_001 && neutral[0] > 0.02);

    // A rotated listener exposes normalize3's zero-vector fallback unless the result is centered.
    let mut coincident = crate::tests::test_listener();
    coincident.position = [4.0, 0.0, 0.0];
    coincident.forward = [1.0, 0.0, 0.0];
    manager.update_listener(coincident.clone()).unwrap();
    refresh_captured_environment(
        &manager,
        source,
        &[coincident.clone()],
        &mut voice,
        &mut replacement,
    );
    assert_eq!(voice.environment_pan, 0.0);
    let mut descriptor = voice.descriptor.clone();
    descriptor.spatial.spatial_scale = None;
    manager.update_source(descriptor).unwrap();
    coincident.position = [0.0, 0.0, 0.0];
    manager.update_listener(coincident.clone()).unwrap();
    let before_rejection = manager.source_environment_snapshot_for_test(source);
    assert!(manager.set_default_spatial_scale(f32::MAX).is_err());
    assert_eq!(manager.default_spatial_scale().unwrap(), 7.0);
    assert_eq!(
        manager.source_environment_snapshot_for_test(source),
        before_rejection
    );
    manager.set_default_spatial_scale(0.0).unwrap();
    refresh_captured_environment(
        &manager,
        source,
        &[coincident],
        &mut voice,
        &mut replacement,
    );
    assert_eq!(voice.environment_pan, 0.0);

    // Inactive automation edits descriptors directly rather than using update_source/listener.
    // Its cache must already be valid when the next output generation clones these voices.
    use zircon_runtime::core::framework::sound::{SoundAutomationTarget, SoundParameterId};
    let mut automated = crate::engine::SoundEngineState::new(&crate::SoundConfig::default());
    automated.graph = Arc::new(send_graph(1.0, 1.0, false, 0.0));
    automated.clips = clips;
    let mut automated_voice = voice;
    automated_voice.kira_playback = None;
    automated.sources.insert(source, automated_voice);
    let automated_listener = crate::tests::test_listener();
    automated
        .listeners
        .insert(automated_listener.id, automated_listener.clone());
    crate::automation::target::apply_automation_target(
        &mut automated,
        SoundAutomationTarget::Source(source),
        &SoundParameterId::new("position_x"),
        2.0,
    )
    .unwrap();
    assert_eq!(
        (
            automated.sources[&source].environment_gain,
            automated.sources[&source].environment_pan
        ),
        (0.5, 1.0)
    );
    crate::automation::target::apply_automation_target(
        &mut automated,
        SoundAutomationTarget::Listener(automated_listener.id),
        &SoundParameterId::new("position_x"),
        4.0,
    )
    .unwrap();
    assert_eq!(
        (
            automated.sources[&source].environment_gain,
            automated.sources[&source].environment_pan
        ),
        (0.5, -1.0)
    );
}

fn spatial_scale_fixture(
    track: SoundTrackId,
    explicit_scale: Option<f32>,
) -> (
    crate::DefaultSoundManager,
    zircon_runtime::core::framework::sound::SoundSourceId,
    std::collections::HashMap<
        zircon_runtime::core::framework::sound::SoundClipId,
        crate::engine::LoadedClip,
    >,
) {
    use zircon_runtime::core::framework::sound::{
        SoundMixerGraphManager, SoundSourceDescriptor, SoundSourceManager,
    };
    let manager = crate::DefaultSoundManager::default();
    if track != SoundTrackId::master() {
        manager
            .configure_mixer(send_graph(1.0, 1.0, false, 0.0))
            .unwrap();
    }
    let asset = crate::tests::test_clip_with_rate(
        "res://sound/spatial-scale-capture.wav",
        48_000,
        &vec![TEST_SIGNAL_AMPLITUDE; 96_000],
    );
    let loaded = crate::engine::LoadedClip::new(asset.clone()).unwrap();
    let clip = manager.insert_clip_for_test(asset);
    let mut descriptor = SoundSourceDescriptor::clip(clip);
    descriptor.output_track = track;
    descriptor.position = [4.0, 0.0, 0.0];
    descriptor.spatial.spatial_blend = 1.0;
    descriptor.spatial.spatial_scale = explicit_scale;
    descriptor.looped = true;
    let source = manager.create_source(descriptor).unwrap();
    (
        manager,
        source,
        std::collections::HashMap::from([(clip, loaded)]),
    )
}

fn refresh_captured_environment(
    manager: &crate::DefaultSoundManager,
    source: zircon_runtime::core::framework::sound::SoundSourceId,
    listeners: &[zircon_runtime::core::framework::sound::SoundListenerDescriptor],
    voice: &mut crate::engine::SourceVoice,
    engine: &mut KiraEngine<CaptureBackend>,
) {
    use zircon_runtime::core::framework::sound::SoundRuntimeSettingsManager;
    let (scale, projected) = manager.source_environment_snapshot_for_test(source);
    assert_eq!(manager.default_spatial_scale().unwrap(), scale);
    let projected = projected.unwrap();
    let binding = voice.kira_playback;
    let cursor = (voice.cursor_frame, voice.cursor_position);
    voice.descriptor = projected.descriptor;
    voice
        .project_environment(scale, listeners, &std::collections::HashMap::new())
        .unwrap();
    assert_eq!(
        (voice.environment_gain, voice.environment_pan),
        (projected.environment_gain, projected.environment_pan)
    );
    voice.apply_environment(engine).unwrap();
    assert_eq!(voice.kira_playback, binding);
    assert_eq!((voice.cursor_frame, voice.cursor_position), cursor);
}

fn render_stable_stereo(
    engine: &mut KiraEngine<CaptureBackend>,
    captures: &Arc<Mutex<Vec<f32>>>,
) -> [f32; 2] {
    for _ in 0..8 {
        engine
            .with_backend_mut(CaptureBackend::render)
            .expect("capture backend must be active");
    }
    captures
        .lock()
        .expect("capture samples must not poison")
        .chunks_exact(2)
        .fold([0.0_f32; 2], |peak, frame| {
            [peak[0].max(frame[0].abs()), peak[1].max(frame[1].abs())]
        })
}
