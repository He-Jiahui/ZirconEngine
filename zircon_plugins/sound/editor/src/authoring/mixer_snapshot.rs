use zircon_runtime::core::framework::sound::{
    SoundMixerSnapshot, SoundTrackDescriptor, SoundTrackId, SoundTrackSend,
};

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SoundMixerAuthoringTrack<'a> {
    /// Position in this borrowed snapshot, never a persistent node or asset ID.
    pub source_index: usize,
    pub track: &'a SoundTrackDescriptor,
}

/// Parent output and auxiliary sends remain distinct even with identical endpoints.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum SoundMixerAuthoringRoute<'a> {
    Parent {
        source_track_index: usize,
        source: SoundTrackId,
        target: SoundTrackId,
    },
    Send {
        source_track_index: usize,
        /// Snapshot-local position only; sends currently have no independent ID.
        source_send_index: usize,
        source: SoundTrackId,
        send: &'a SoundTrackSend,
    },
}

/// A lossless read view of the caller's mixer snapshot, without another audio store.
///
/// This does not validate or compile the graph, resolve an asset document, or prove
/// output readiness. The caller retains project/source revision and provider scope.
#[derive(Debug)]
pub struct SoundMixerAuthoringSnapshot<'a> {
    /// Includes original meters, effects, sources, automation and dynamic events.
    /// Missing meters stay absent; this view never synthesizes silent/ready state.
    pub source: &'a SoundMixerSnapshot,
    pub tracks: Vec<SoundMixerAuthoringTrack<'a>>,
    pub routes: Vec<SoundMixerAuthoringRoute<'a>>,
}

impl<'a> SoundMixerAuthoringSnapshot<'a> {
    pub fn new(source: &'a SoundMixerSnapshot) -> Self {
        let tracks = source
            .graph
            .tracks
            .iter()
            .enumerate()
            .map(|(source_index, track)| SoundMixerAuthoringTrack {
                source_index,
                track,
            })
            .collect();
        let mut routes = Vec::new();
        for (source_track_index, track) in source.graph.tracks.iter().enumerate() {
            if let Some(target) = track.parent {
                routes.push(SoundMixerAuthoringRoute::Parent {
                    source_track_index,
                    source: track.id,
                    target,
                });
            }
            routes.extend(
                track
                    .sends
                    .iter()
                    .enumerate()
                    .map(|(source_send_index, send)| SoundMixerAuthoringRoute::Send {
                        source_track_index,
                        source_send_index,
                        source: track.id,
                        send,
                    }),
            );
        }

        // Do not key these rows by ID or endpoints: duplicate IDs, repeated sends
        // and missing targets must remain visible to the existing validator/UI.
        Self {
            source,
            tracks,
            routes,
        }
    }
}
