use zircon_runtime::asset::{AssetReference, AssetUri};
use zircon_runtime::core::framework::animation::{
    AnimationConditionOperatorAsset, AnimationStateAsset, AnimationStateMachineAsset,
    AnimationStateTransitionAsset, AnimationTransitionConditionAsset,
};

use super::parameters::parse_parameter_value;
use super::support::frame_to_seconds;
use super::{AnimationEditorSession, DEFAULT_STATE_MACHINE_TRANSITION_FPS};

impl AnimationEditorSession {
    pub fn create_state(&mut self, state_name: &str, graph_locator: &str) -> Result<bool, String> {
        let asset = self.state_machine_asset_mut()?;
        if asset.states.iter().any(|state| state.name == state_name) {
            return Ok(false);
        }
        let graph = AssetReference::from_locator(
            AssetUri::parse(graph_locator).map_err(|error| error.to_string())?,
        );
        asset
            .states
            .push(AnimationStateAsset::graph_ref(state_name, graph));
        self.dirty = true;
        Ok(true)
    }

    pub fn remove_state(&mut self, state_name: &str) -> Result<bool, String> {
        let asset = self.state_machine_asset_mut()?;
        let before = asset.states.len();
        asset.states.retain(|state| state.name != state_name);
        if before == asset.states.len() {
            return Ok(false);
        }
        asset.transitions.retain(|transition| {
            transition.from_state != state_name && transition.to_state != state_name
        });
        if asset.entry_state == state_name {
            asset.entry_state = asset
                .states
                .first()
                .map(|state| state.name.clone())
                .unwrap_or_default();
        }
        self.dirty = true;
        Ok(true)
    }

    pub fn set_entry_state(&mut self, state_name: &str) -> Result<bool, String> {
        let asset = self.state_machine_asset_mut()?;
        if asset.entry_state == state_name {
            return Ok(false);
        }
        if !state_machine_has_state(asset, state_name) {
            return Ok(false);
        }
        asset.entry_state = state_name.to_string();
        self.dirty = true;
        Ok(true)
    }

    pub fn create_transition(
        &mut self,
        from_state: &str,
        to_state: &str,
        duration_frames: u32,
    ) -> Result<bool, String> {
        let asset = self.state_machine_asset_mut()?;
        if !state_machine_has_state(asset, from_state) || !state_machine_has_state(asset, to_state)
        {
            return Ok(false);
        }
        if let Some(transition) = asset.transitions.iter_mut().find(|transition| {
            transition.from_state == from_state && transition.to_state == to_state
        }) {
            let duration_seconds =
                frame_to_seconds(duration_frames, DEFAULT_STATE_MACHINE_TRANSITION_FPS);
            let changed = (transition.duration_seconds - duration_seconds).abs() > f32::EPSILON;
            transition.duration_seconds = duration_seconds;
            self.dirty |= changed;
            return Ok(changed);
        }
        asset.transitions.push(AnimationStateTransitionAsset {
            from_state: from_state.to_string(),
            to_state: to_state.to_string(),
            duration_seconds: frame_to_seconds(
                duration_frames,
                DEFAULT_STATE_MACHINE_TRANSITION_FPS,
            ),
            exit_time: None,
            interruption: Default::default(),
            conditions: Vec::new(),
        });
        self.dirty = true;
        Ok(true)
    }

    pub fn remove_transition(&mut self, from_state: &str, to_state: &str) -> Result<bool, String> {
        let asset = self.state_machine_asset_mut()?;
        let before = asset.transitions.len();
        asset.transitions.retain(|transition| {
            !(transition.from_state == from_state && transition.to_state == to_state)
        });
        let changed = before != asset.transitions.len();
        self.dirty |= changed;
        Ok(changed)
    }

    pub fn set_transition_condition(
        &mut self,
        from_state: &str,
        to_state: &str,
        parameter_name: &str,
        operator: &str,
        value_literal: &str,
    ) -> Result<bool, String> {
        let asset = self.state_machine_asset_mut()?;
        if !state_machine_has_state(asset, from_state) || !state_machine_has_state(asset, to_state)
        {
            return Ok(false);
        }
        let Some(transition_index) = asset.transitions.iter().position(|transition| {
            transition.from_state == from_state && transition.to_state == to_state
        }) else {
            return Ok(false);
        };
        let Some(operator) = parse_condition_operator(operator) else {
            return Ok(false);
        };
        let transition = &mut asset.transitions[transition_index];
        let existing_value = transition
            .conditions
            .iter()
            .find(|condition| condition.parameter == parameter_name)
            .and_then(|condition| condition.value.clone());
        let Some(value) = parse_parameter_value(existing_value.as_ref(), value_literal) else {
            return Ok(false);
        };
        let next_condition = AnimationTransitionConditionAsset {
            parameter: parameter_name.to_string(),
            operator,
            value: Some(value),
        };
        if let Some(condition) = transition
            .conditions
            .iter_mut()
            .find(|condition| condition.parameter == parameter_name)
        {
            let changed = *condition != next_condition;
            *condition = next_condition;
            self.dirty |= changed;
            return Ok(changed);
        }
        transition.conditions.push(next_condition);
        self.dirty = true;
        Ok(true)
    }
}

pub(super) fn transition_label(transition: &AnimationStateTransitionAsset) -> String {
    format!("{} -> {}", transition.from_state, transition.to_state)
}

fn state_machine_has_state(asset: &AnimationStateMachineAsset, state_name: &str) -> bool {
    asset.states.iter().any(|state| state.name == state_name)
}

fn parse_condition_operator(operator: &str) -> Option<AnimationConditionOperatorAsset> {
    match operator {
        "equal" => Some(AnimationConditionOperatorAsset::Equal),
        "not_equal" => Some(AnimationConditionOperatorAsset::NotEqual),
        "greater" => Some(AnimationConditionOperatorAsset::Greater),
        "greater_equal" => Some(AnimationConditionOperatorAsset::GreaterEqual),
        "less" => Some(AnimationConditionOperatorAsset::Less),
        "less_equal" => Some(AnimationConditionOperatorAsset::LessEqual),
        "triggered" => Some(AnimationConditionOperatorAsset::Triggered),
        _ => None,
    }
}
