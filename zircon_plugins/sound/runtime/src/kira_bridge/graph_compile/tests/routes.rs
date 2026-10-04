use super::*;

#[test]
fn shared_downstream_routes_are_reused_from_cache() {
    let a = SoundTrackId::new(1);
    let b = SoundTrackId::new(2);
    let c = SoundTrackId::new(3);
    let d = SoundTrackId::new(4);
    let mut tracks = [
        track(a, &[(b, 1.0), (c, 2.0)]),
        track(b, &[(d, 0.5)]),
        track(c, &[(d, 0.25)]),
        track(d, &[]),
    ];
    for track in &mut tracks {
        track.parent = None;
    }
    let lookup = tracks.iter().map(|track| (track.id, track)).collect();

    let expanded = expanded_post_effect_sends(&lookup).unwrap();

    assert_eq!(
        expanded.get(&a).map(Vec::as_slice),
        Some([send(b, 1.0), send(c, 2.0), send(d, 1.0),].as_slice())
    );
    assert_eq!(
        expanded.get(&b).map(Vec::as_slice),
        Some([send(d, 0.5)].as_slice())
    );
    assert_eq!(
        expanded.get(&c).map(Vec::as_slice),
        Some([send(d, 0.25)].as_slice())
    );
    assert_eq!(expanded.get(&d).map(Vec::as_slice), Some([].as_slice()));
}

fn track(id: SoundTrackId, sends: &[(SoundTrackId, f32)]) -> SoundTrackDescriptor {
    let mut track = SoundTrackDescriptor::child(id, format!("Track {}", id.raw()));
    track.sends = sends
        .iter()
        .map(|(target, gain)| send(*target, *gain))
        .collect();
    track
}

fn send(target: SoundTrackId, gain: f32) -> SoundTrackSend {
    SoundTrackSend {
        target,
        gain,
        pre_effects: false,
    }
}
