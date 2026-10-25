use crate::{SoundMixerAuthoringRoute, SoundMixerAuthoringSnapshot};
use zircon_runtime::core::framework::sound::{
    SoundEffectDescriptor, SoundEffectId, SoundEffectKind, SoundGainEffect, SoundMixerGraph,
    SoundMixerSnapshot, SoundRayTracingConvolutionStatus, SoundTrackDescriptor, SoundTrackId,
    SoundTrackMeter, SoundTrackSend,
};

fn snapshot(graph: SoundMixerGraph) -> SoundMixerSnapshot {
    SoundMixerSnapshot {
        graph,
        meters: Vec::new(),
        latency_frames: 0,
        ray_tracing: SoundRayTracingConvolutionStatus::Disabled,
    }
}

#[test]
fn mixer_authoring_preserves_duplicate_ids_and_unresolved_parent() {
    let mut graph = SoundMixerGraph::default_stereo(48_000);
    let mut first = SoundTrackDescriptor::child(SoundTrackId::new(8), "中文轨道");
    first.parent = Some(SoundTrackId::new(99));
    graph.tracks.push(first);
    graph.tracks.push(SoundTrackDescriptor::child(
        SoundTrackId::new(8),
        "A second track with the same runtime ID",
    ));
    let source = snapshot(graph);
    let view = SoundMixerAuthoringSnapshot::new(&source);

    assert_eq!(view.tracks.len(), 3);
    assert_eq!(view.tracks[1].source_index, 1);
    assert_eq!(view.tracks[2].source_index, 2);
    assert_eq!(view.tracks[1].track.id, view.tracks[2].track.id);
    assert_eq!(view.tracks[1].track.display_name, "中文轨道");
    assert!(std::ptr::eq(view.source, &source));
    assert!(std::ptr::eq(view.tracks[1].track, &source.graph.tracks[1]));
    assert_eq!(
        view.routes[0],
        SoundMixerAuthoringRoute::Parent {
            source_track_index: 1,
            source: SoundTrackId::new(8),
            target: SoundTrackId::new(99),
        }
    );
}

#[test]
fn mixer_authoring_keeps_parent_and_parallel_sends_distinct() {
    let mut graph = SoundMixerGraph::default_stereo(48_000);
    let mut track = SoundTrackDescriptor::child(SoundTrackId::new(8), "Music");
    track.sends = vec![
        SoundTrackSend {
            target: SoundTrackId::master(),
            gain: 0.25,
            pre_effects: false,
        },
        SoundTrackSend {
            target: SoundTrackId::master(),
            gain: 0.75,
            pre_effects: true,
        },
    ];
    graph.tracks.push(track);
    let source = snapshot(graph);
    let view = SoundMixerAuthoringSnapshot::new(&source);

    assert_eq!(view.routes.len(), 3);
    assert!(matches!(
        view.routes[0],
        SoundMixerAuthoringRoute::Parent { .. }
    ));
    for source_send_index in 0..2 {
        let SoundMixerAuthoringRoute::Send {
            source_track_index,
            source: source_track,
            source_send_index: actual_index,
            send,
        } = view.routes[source_send_index + 1]
        else {
            panic!("parent output and both original sends must remain separate");
        };
        assert_eq!(source_track_index, 1);
        assert_eq!(source_track, SoundTrackId::new(8));
        assert_eq!(actual_index, source_send_index);
        assert!(std::ptr::eq(
            send,
            &source.graph.tracks[1].sends[source_send_index]
        ));
        assert_eq!(send.pre_effects, source_send_index == 1);
    }
    assert!(view.source.meters.is_empty());
}

#[test]
fn mixer_authoring_preserves_effect_order_controls_and_unmatched_meter() {
    let mut graph = SoundMixerGraph::default_stereo(48_000);
    let mut track = SoundTrackDescriptor::child(SoundTrackId::new(8), "Uncompiled effects");
    track.controls.pan = 0.5;
    for id in [7, 3] {
        track.effects.push(SoundEffectDescriptor::new(
            SoundEffectId::new(id),
            format!("Effect {id}"),
            SoundEffectKind::Gain(SoundGainEffect { gain: 0.5 }),
        ));
    }
    graph.tracks.push(track);
    let mut source = snapshot(graph);
    source.meters.push(SoundTrackMeter {
        track: SoundTrackId::new(99),
        peak_left: 0.5,
        peak_right: 0.25,
        rms_left: 0.125,
        rms_right: 0.0625,
    });
    let before = source.clone();
    let view = SoundMixerAuthoringSnapshot::new(&source);

    assert_eq!(
        view.tracks[1]
            .track
            .effects
            .iter()
            .map(|effect| effect.id)
            .collect::<Vec<_>>(),
        [SoundEffectId::new(7), SoundEffectId::new(3)]
    );
    assert_eq!(view.tracks[1].track.controls.pan, 0.5);
    assert_eq!(view.source.meters[0].track, SoundTrackId::new(99));
    assert_eq!(source, before);
}
