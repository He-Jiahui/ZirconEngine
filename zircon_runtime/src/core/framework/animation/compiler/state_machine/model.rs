//! Immutable state-machine compiler products.

use crate::core::framework::animation::{
    AnimationConditionOperatorAsset, AnimationParameterValue,
    AnimationStateMachineLayerBlendModeAsset, AnimationTransitionInterruptionPolicyAsset,
};
use crate::core::math::{Real, Vec2};
use crate::core::resource::AssetReference;

use super::super::{AnimationCompileDiagnostic, AnimationCompiledParameter};

/// One blend-space sample with a validated 1D coordinate.
#[derive(Clone, Debug, PartialEq)]
pub struct AnimationCompiledBlendSpace1DSample {
    pub position: Real,
    pub graph: AssetReference,
}

/// One blend-space sample with a validated 2D coordinate.
#[derive(Clone, Debug, PartialEq)]
pub struct AnimationCompiledBlendSpace2DSample {
    pub position: Vec2,
    pub graph: AssetReference,
}

/// A state kind whose parameter references have been resolved to parameter slots.
#[derive(Clone, Debug, PartialEq)]
pub enum AnimationCompiledStateKind {
    Clip {
        clip: AssetReference,
    },
    BlendSpace1D {
        parameter: usize,
        samples: Vec<AnimationCompiledBlendSpace1DSample>,
    },
    BlendSpace2D {
        parameter: usize,
        samples: Vec<AnimationCompiledBlendSpace2DSample>,
    },
    SubMachine {
        state_machine: AssetReference,
    },
    GraphRef {
        graph: AssetReference,
    },
}

/// A stable state slot retained in source order.
#[derive(Clone, Debug, PartialEq)]
pub struct AnimationCompiledState {
    name: String,
    kind: AnimationCompiledStateKind,
}

impl AnimationCompiledState {
    pub(super) fn new(name: String, kind: AnimationCompiledStateKind) -> Self {
        Self { name, kind }
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn kind(&self) -> &AnimationCompiledStateKind {
        &self.kind
    }
}

/// A transition condition whose parameter link has been resolved to a parameter slot.
#[derive(Clone, Debug, PartialEq)]
pub struct AnimationCompiledTransitionCondition {
    parameter: usize,
    operator: AnimationConditionOperatorAsset,
    value: Option<AnimationParameterValue>,
}

impl AnimationCompiledTransitionCondition {
    pub(super) fn new(
        parameter: usize,
        operator: AnimationConditionOperatorAsset,
        value: Option<AnimationParameterValue>,
    ) -> Self {
        Self {
            parameter,
            operator,
            value,
        }
    }

    pub fn parameter(&self) -> usize {
        self.parameter
    }

    pub fn operator(&self) -> AnimationConditionOperatorAsset {
        self.operator
    }

    pub fn value(&self) -> Option<&AnimationParameterValue> {
        self.value.as_ref()
    }
}

/// A transition with resolved state and parameter slots.
#[derive(Clone, Debug, PartialEq)]
pub struct AnimationCompiledTransition {
    from_state: usize,
    to_state: usize,
    duration_seconds: Real,
    exit_time: Option<Real>,
    interruption: AnimationTransitionInterruptionPolicyAsset,
    conditions: Vec<AnimationCompiledTransitionCondition>,
}

impl AnimationCompiledTransition {
    pub(super) fn new(
        from_state: usize,
        to_state: usize,
        duration_seconds: Real,
        exit_time: Option<Real>,
        interruption: AnimationTransitionInterruptionPolicyAsset,
        conditions: Vec<AnimationCompiledTransitionCondition>,
    ) -> Self {
        Self {
            from_state,
            to_state,
            duration_seconds,
            exit_time,
            interruption,
            conditions,
        }
    }

    pub fn from_state(&self) -> usize {
        self.from_state
    }

    pub fn to_state(&self) -> usize {
        self.to_state
    }

    pub fn duration_seconds(&self) -> Real {
        self.duration_seconds
    }

    pub fn exit_time(&self) -> Option<Real> {
        self.exit_time
    }

    pub fn interruption(&self) -> AnimationTransitionInterruptionPolicyAsset {
        self.interruption
    }

    pub fn conditions(&self) -> &[AnimationCompiledTransitionCondition] {
        &self.conditions
    }
}

/// A validated state-machine layer. Its referenced machine remains an external dependency.
#[derive(Clone, Debug, PartialEq)]
pub struct AnimationCompiledStateMachineLayer {
    name: String,
    state_machine: AssetReference,
    weight: Real,
    blend_mode: AnimationStateMachineLayerBlendModeAsset,
    mask_weights: Vec<Real>,
}

impl AnimationCompiledStateMachineLayer {
    pub(super) fn new(
        name: String,
        state_machine: AssetReference,
        weight: Real,
        blend_mode: AnimationStateMachineLayerBlendModeAsset,
        mask_weights: Vec<Real>,
    ) -> Self {
        Self {
            name,
            state_machine,
            weight,
            blend_mode,
            mask_weights,
        }
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn state_machine(&self) -> &AssetReference {
        &self.state_machine
    }

    pub fn weight(&self) -> Real {
        self.weight
    }

    pub fn blend_mode(&self) -> AnimationStateMachineLayerBlendModeAsset {
        self.blend_mode
    }

    pub fn mask_weights(&self) -> &[Real] {
        &self.mask_weights
    }
}

/// A validated state-machine IR with source-order-stable dense slots.
#[derive(Clone, Debug, PartialEq)]
pub struct AnimationCompiledStateMachine {
    parameters: Vec<AnimationCompiledParameter>,
    states: Vec<AnimationCompiledState>,
    entry_state: usize,
    transitions: Vec<AnimationCompiledTransition>,
    layers: Vec<AnimationCompiledStateMachineLayer>,
}

impl AnimationCompiledStateMachine {
    pub(super) fn new(
        parameters: Vec<AnimationCompiledParameter>,
        states: Vec<AnimationCompiledState>,
        entry_state: usize,
        transitions: Vec<AnimationCompiledTransition>,
        layers: Vec<AnimationCompiledStateMachineLayer>,
    ) -> Self {
        Self {
            parameters,
            states,
            entry_state,
            transitions,
            layers,
        }
    }

    pub fn parameters(&self) -> &[AnimationCompiledParameter] {
        &self.parameters
    }

    pub fn states(&self) -> &[AnimationCompiledState] {
        &self.states
    }

    pub fn entry_state(&self) -> usize {
        self.entry_state
    }

    pub fn transitions(&self) -> &[AnimationCompiledTransition] {
        &self.transitions
    }

    pub fn layers(&self) -> &[AnimationCompiledStateMachineLayer] {
        &self.layers
    }
}

/// Result of compiling a state-machine asset without loading external resources.
#[derive(Clone, Debug, PartialEq)]
pub struct AnimationStateMachineCompilation {
    artifact: Option<AnimationCompiledStateMachine>,
    diagnostics: Vec<AnimationCompileDiagnostic>,
}

impl AnimationStateMachineCompilation {
    pub(crate) fn new(
        artifact: Option<AnimationCompiledStateMachine>,
        diagnostics: Vec<AnimationCompileDiagnostic>,
    ) -> Self {
        Self {
            artifact,
            diagnostics,
        }
    }

    pub fn artifact(&self) -> Option<&AnimationCompiledStateMachine> {
        self.artifact.as_ref()
    }

    pub fn diagnostics(&self) -> &[AnimationCompileDiagnostic] {
        &self.diagnostics
    }
}
