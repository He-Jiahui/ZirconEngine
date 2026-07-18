use zircon_runtime::core::framework::sound::{SoundAutomationTarget, SoundError, SoundParameterId};

use crate::automation::values::ensure_finite_value;
use crate::descriptor_validation::listener::validate_listener_descriptor;
use crate::descriptor_validation::source::validate_source_descriptor;
use crate::descriptor_validation::volume::validate_volume_descriptor;
use crate::engine::SoundEngineState;
use crate::kira_bridge::validate_graph;

use super::{effect, listener, parameter_values, source, track, volume};

pub(crate) fn apply_automation_target(
    state: &mut SoundEngineState,
    target: SoundAutomationTarget,
    parameter: &SoundParameterId,
    value: f32,
) -> Result<(), SoundError> {
    ensure_automation_execution_available(state.kira.is_active())?;
    ensure_finite_value("sound automation value", value)?;
    match target {
        SoundAutomationTarget::Track(track) => {
            let mut graph = (*state.graph).clone();
            let track_descriptor = graph
                .tracks
                .iter_mut()
                .find(|candidate| candidate.id == track)
                .ok_or(SoundError::UnknownTrack { track })?;
            track::apply_track_parameter(track_descriptor, parameter, value)?;
            validate_graph(&graph)?;
            state.replace_graph(graph);
            Ok(())
        }
        SoundAutomationTarget::Effect { track, effect } => {
            let mut graph = (*state.graph).clone();
            let track_descriptor = graph
                .tracks
                .iter_mut()
                .find(|candidate| candidate.id == track)
                .ok_or(SoundError::UnknownTrack { track })?;
            let effect_descriptor = track_descriptor
                .effects
                .iter_mut()
                .find(|candidate| candidate.id == effect)
                .ok_or(SoundError::UnknownEffect { effect })?;
            effect::apply_effect_parameter(effect_descriptor, parameter, value)?;
            validate_graph(&graph)?;
            state.replace_graph(graph);
            Ok(())
        }
        SoundAutomationTarget::Source(source_id) => {
            let mut descriptor = state
                .sources
                .get(&source_id)
                .ok_or(SoundError::UnknownSource { source_id })?
                .descriptor
                .clone();
            source::apply_source_parameter(&mut descriptor, parameter, value)?;
            validate_source_descriptor(state, &descriptor)?;
            state
                .sources
                .get_mut(&source_id)
                .ok_or(SoundError::UnknownSource { source_id })?
                .descriptor = descriptor;
            Ok(())
        }
        SoundAutomationTarget::Listener(listener) => {
            let mut descriptor = state
                .listeners
                .get(&listener)
                .ok_or(SoundError::UnknownListener { listener })?
                .clone();
            listener::apply_listener_parameter(&mut descriptor, parameter, value)?;
            validate_listener_descriptor(state, &descriptor)?;
            state.listeners.insert(listener, descriptor);
            Ok(())
        }
        SoundAutomationTarget::Volume(volume) => {
            let mut descriptor = state
                .volumes
                .get(&volume)
                .ok_or(SoundError::UnknownVolume { volume })?
                .clone();
            volume::apply_volume_parameter(&mut descriptor, parameter, value)?;
            validate_volume_descriptor(&descriptor)?;
            state.volumes.insert(volume, descriptor);
            Ok(())
        }
        SoundAutomationTarget::SynthParameter(target_parameter) => {
            if parameter.as_str() != "value" && parameter.as_str() != target_parameter.as_str() {
                return Err(parameter_values::unsupported_automation_parameter(
                    "synth parameter",
                    parameter,
                ));
            }
            state.parameters.insert(target_parameter, value);
            Ok(())
        }
    }
}

pub(crate) fn ensure_automation_execution_available(kira_active: bool) -> Result<(), SoundError> {
    if kira_active {
        return Err(SoundError::UnsupportedAdvancedFeature(
            "active sound automation execution is enabled by Sound M5".to_string(),
        ));
    }
    Ok(())
}
