use std::collections::{HashMap, HashSet};

use zircon_runtime::core::framework::sound::{
    SoundEffectDescriptor, SoundEffectKind, SoundError, SoundMixerGraph, SoundTrackDescriptor,
    SoundTrackId, SoundTrackSend,
};

pub(crate) fn validate_graph(graph: &SoundMixerGraph) -> Result<(), SoundError> {
    if graph.master_track().is_none() {
        return Err(SoundError::InvalidMixerGraph(
            "mixer graph must contain the master track".to_string(),
        ));
    }
    let track_ids = graph
        .tracks
        .iter()
        .map(|track| track.id)
        .collect::<HashSet<_>>();
    if track_ids.len() != graph.tracks.len() {
        return Err(SoundError::InvalidMixerGraph(
            "mixer graph contains duplicate track ids".to_string(),
        ));
    }
    for track in &graph.tracks {
        validate_track_controls(track)?;
        if let Some(parent) = track.parent {
            if !track_ids.contains(&parent) {
                return Err(SoundError::UnknownTrack { track: parent });
            }
            if parent == track.id {
                return Err(SoundError::InvalidMixerGraph(
                    "track cannot route to itself".to_string(),
                ));
            }
        }
        for send in &track.sends {
            validate_track_send(track, send)?;
            if !track_ids.contains(&send.target) {
                return Err(SoundError::UnknownTrack { track: send.target });
            }
            if send.target == track.id {
                return Err(SoundError::InvalidMixerGraph(
                    "track send cannot route to itself".to_string(),
                ));
            }
        }
        for effect in &track.effects {
            validate_effect_references(track.id, effect, &track_ids)?;
            validate_effect(effect)?;
        }
    }

    let order = topological_track_order(graph)?;
    if order.len() != graph.tracks.len() {
        return Err(SoundError::InvalidMixerGraph(
            "track routing contains a cycle".to_string(),
        ));
    }
    Ok(())
}

pub(crate) fn validate_effect(effect: &SoundEffectDescriptor) -> Result<(), SoundError> {
    if !effect.wet.is_finite() || !(0.0..=1.0).contains(&effect.wet) {
        return Err(SoundError::InvalidEffect(format!(
            "effect {} wet mix must be finite and between 0 and 1",
            effect.display_name
        )));
    }
    match &effect.kind {
        SoundEffectKind::Gain(gain) => validate_finite_effect_value(effect, "gain", gain.gain),
        SoundEffectKind::Filter(filter) => {
            if !filter.cutoff_hz.is_finite()
                || filter.cutoff_hz <= 0.0
                || !filter.resonance.is_finite()
                || filter.resonance < 0.0
                || !filter.gain_db.is_finite()
            {
                return Err(SoundError::InvalidEffect(
                    "filter cutoff, resonance, and gain must be finite, with positive cutoff and non-negative resonance"
                        .to_string(),
                ));
            }
            Ok(())
        }
        SoundEffectKind::Reverb(reverb) => {
            validate_unit_effect_value(effect, "room size", reverb.room_size)?;
            validate_unit_effect_value(effect, "damping", reverb.damping)
        }
        SoundEffectKind::ConvolutionReverb(_) => Ok(()),
        SoundEffectKind::Compressor(compressor) => {
            validate_finite_effect_value(effect, "threshold dB", compressor.threshold_db)?;
            validate_finite_effect_value(effect, "ratio", compressor.ratio)?;
            if compressor.ratio < 1.0 {
                return Err(SoundError::InvalidEffect(
                    "compressor ratio must be at least 1".to_string(),
                ));
            }
            validate_non_negative_effect_value(effect, "attack ms", compressor.attack_ms)?;
            validate_non_negative_effect_value(effect, "release ms", compressor.release_ms)?;
            validate_finite_effect_value(effect, "makeup gain dB", compressor.makeup_gain_db)
        }
        SoundEffectKind::WaveShaper(shaper) => {
            validate_non_negative_effect_value(effect, "drive", shaper.drive)
        }
        SoundEffectKind::Flanger(flanger) => {
            validate_non_negative_effect_value(effect, "rate Hz", flanger.rate_hz)?;
            validate_feedback_effect_value(effect, "feedback", flanger.feedback)
        }
        SoundEffectKind::Phaser(phaser) => {
            validate_non_negative_effect_value(effect, "rate Hz", phaser.rate_hz)?;
            validate_unit_effect_value(effect, "depth", phaser.depth)?;
            validate_feedback_effect_value(effect, "feedback", phaser.feedback)?;
            validate_finite_effect_value(effect, "phase offset", phaser.phase_offset)
        }
        SoundEffectKind::Chorus(chorus) => {
            if chorus.voices == 0 {
                return Err(SoundError::InvalidEffect(
                    "chorus must have at least one voice".to_string(),
                ));
            }
            validate_non_negative_effect_value(effect, "rate Hz", chorus.rate_hz)
        }
        SoundEffectKind::Delay(delay) => {
            validate_feedback_effect_value(effect, "feedback", delay.feedback)
        }
        SoundEffectKind::PanStereo(pan) => {
            validate_pan_value("stereo pan", pan.pan).map_err(SoundError::InvalidEffect)?;
            validate_non_negative_effect_value(effect, "stereo width", pan.width)?;
            validate_finite_effect_value(effect, "left gain", pan.left_gain)?;
            validate_finite_effect_value(effect, "right gain", pan.right_gain)
        }
        SoundEffectKind::Limiter(limiter) => {
            if !limiter.ceiling.is_finite() || limiter.ceiling <= 0.0 {
                return Err(SoundError::InvalidEffect(
                    "limiter ceiling must be finite and positive".to_string(),
                ));
            }
            Ok(())
        }
    }
}

fn validate_track_controls(track: &SoundTrackDescriptor) -> Result<(), SoundError> {
    let controls = track.controls;
    let gain = controls.gain;
    let pan = controls.pan;
    let left_gain = controls.left_gain;
    let right_gain = controls.right_gain;
    if !gain.is_finite() || !left_gain.is_finite() || !right_gain.is_finite() {
        return Err(SoundError::InvalidMixerGraph(format!(
            "track {} controls gain and L/R trims must be finite",
            track.display_name
        )));
    }
    validate_pan_value("track pan", pan).map_err(SoundError::InvalidMixerGraph)
}

fn validate_track_send(
    track: &SoundTrackDescriptor,
    send: &SoundTrackSend,
) -> Result<(), SoundError> {
    if send.gain.is_finite() {
        return Ok(());
    }
    Err(SoundError::InvalidMixerGraph(format!(
        "track {} send gain to {:?} must be finite",
        track.display_name, send.target
    )))
}

fn validate_finite_effect_value(
    effect: &SoundEffectDescriptor,
    label: &str,
    value: f32,
) -> Result<(), SoundError> {
    if value.is_finite() {
        Ok(())
    } else {
        Err(SoundError::InvalidEffect(format!(
            "effect {} {label} must be finite",
            effect.display_name
        )))
    }
}

fn validate_non_negative_effect_value(
    effect: &SoundEffectDescriptor,
    label: &str,
    value: f32,
) -> Result<(), SoundError> {
    validate_finite_effect_value(effect, label, value)?;
    if value >= 0.0 {
        Ok(())
    } else {
        Err(SoundError::InvalidEffect(format!(
            "effect {} {label} must be non-negative",
            effect.display_name
        )))
    }
}

fn validate_unit_effect_value(
    effect: &SoundEffectDescriptor,
    label: &str,
    value: f32,
) -> Result<(), SoundError> {
    validate_finite_effect_value(effect, label, value)?;
    if (0.0..=1.0).contains(&value) {
        Ok(())
    } else {
        Err(SoundError::InvalidEffect(format!(
            "effect {} {label} must be between 0 and 1",
            effect.display_name
        )))
    }
}

fn validate_feedback_effect_value(
    effect: &SoundEffectDescriptor,
    label: &str,
    value: f32,
) -> Result<(), SoundError> {
    validate_finite_effect_value(effect, label, value)?;
    if (0.0..=0.99).contains(&value) {
        Ok(())
    } else {
        Err(SoundError::InvalidEffect(format!(
            "effect {} {label} must be between 0 and 0.99",
            effect.display_name
        )))
    }
}

fn validate_pan_value(label: &str, value: f32) -> Result<(), String> {
    if value.is_finite() && (-1.0..=1.0).contains(&value) {
        Ok(())
    } else {
        Err(format!("{label} must be finite and between -1 and 1"))
    }
}

pub(crate) fn track_render_order(graph: &SoundMixerGraph) -> Vec<SoundTrackId> {
    topological_track_order(graph)
        .unwrap_or_else(|_| graph.tracks.iter().map(|track| track.id).collect())
}

fn validate_effect_references(
    track: SoundTrackId,
    effect: &SoundEffectDescriptor,
    track_ids: &HashSet<SoundTrackId>,
) -> Result<(), SoundError> {
    if let SoundEffectKind::Compressor(compressor) = &effect.kind {
        if let Some(sidechain) = compressor.sidechain {
            if !track_ids.contains(&sidechain.track) {
                return Err(SoundError::UnknownTrack {
                    track: sidechain.track,
                });
            }
            if !sidechain.pre_effects && sidechain.track == track {
                return Err(SoundError::InvalidMixerGraph(
                    "post-effect sidechain cannot read from the same track".to_string(),
                ));
            }
        }
    }
    Ok(())
}

fn topological_track_order(graph: &SoundMixerGraph) -> Result<Vec<SoundTrackId>, SoundError> {
    let track_ids = graph
        .tracks
        .iter()
        .map(|track| track.id)
        .collect::<Vec<_>>();
    let mut outgoing = track_ids
        .iter()
        .copied()
        .map(|track| (track, Vec::new()))
        .collect::<HashMap<_, _>>();
    let mut indegree = track_ids
        .iter()
        .copied()
        .map(|track| (track, 0_usize))
        .collect::<HashMap<_, _>>();

    for (source, target) in render_dependencies(graph) {
        outgoing.entry(source).or_default().push(target);
        *indegree.entry(target).or_default() += 1;
    }

    let mut ready = track_ids
        .iter()
        .copied()
        .filter(|track| indegree.get(track).copied().unwrap_or_default() == 0)
        .collect::<Vec<_>>();
    let mut order = Vec::with_capacity(track_ids.len());

    while let Some(track) = ready.first().copied() {
        ready.remove(0);
        order.push(track);
        if let Some(targets) = outgoing.get(&track) {
            for target in targets {
                let Some(target_indegree) = indegree.get_mut(target) else {
                    continue;
                };
                *target_indegree = target_indegree.saturating_sub(1);
                if *target_indegree == 0 {
                    ready.push(*target);
                    ready.sort_by_key(|candidate| {
                        track_ids
                            .iter()
                            .position(|track| track == candidate)
                            .unwrap_or(usize::MAX)
                    });
                }
            }
        }
    }

    if order.len() == track_ids.len() {
        Ok(order)
    } else {
        Err(SoundError::InvalidMixerGraph(
            "track routing contains a cycle".to_string(),
        ))
    }
}

fn render_dependencies(graph: &SoundMixerGraph) -> Vec<(SoundTrackId, SoundTrackId)> {
    let mut edges = Vec::new();
    for track in &graph.tracks {
        if let Some(parent) = track.parent {
            edges.push((track.id, parent));
        }
        for send in &track.sends {
            edges.push((track.id, send.target));
        }
        for effect in &track.effects {
            if let SoundEffectKind::Compressor(compressor) = &effect.kind {
                if let Some(sidechain) = compressor.sidechain {
                    if !sidechain.pre_effects {
                        edges.push((sidechain.track, track.id));
                    }
                }
            }
        }
    }
    edges
}
