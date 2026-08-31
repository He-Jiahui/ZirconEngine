//! Immutable builtin schema facts for animation authoring compilation.

/// Asset families accepted by the shared animation compiler.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AnimationCompilerAssetKind {
    Sequence,
    Graph,
    StateMachine,
}

/// Stable version for compiler-owned schemas and future plugin contributions.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct AnimationCompilerSchemaVersion {
    major: u16,
    minor: u16,
}

impl AnimationCompilerSchemaVersion {
    pub const fn new(major: u16, minor: u16) -> Self {
        Self { major, minor }
    }

    pub const fn major(self) -> u16 {
        self.major
    }

    pub const fn minor(self) -> u16 {
        self.minor
    }
}

/// Ownership and compatibility identity of a compiler schema contribution.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct AnimationCompilerSchemaOwner {
    id: &'static str,
    version: AnimationCompilerSchemaVersion,
}

impl AnimationCompilerSchemaOwner {
    pub const fn id(self) -> &'static str {
        self.id
    }

    pub const fn version(self) -> AnimationCompilerSchemaVersion {
        self.version
    }
}

/// Graph node identities supported by the current canonical asset schema.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AnimationGraphNodeSchemaKind {
    Clip,
    Blend,
    Additive,
    Mask,
    Output,
}

impl AnimationGraphNodeSchemaKind {
    /// Resolves a schema-defined graph node identifier without involving any UI capability gate.
    pub fn from_id(id: &str) -> Option<Self> {
        match id {
            "clip" => Some(Self::Clip),
            "blend" => Some(Self::Blend),
            "additive" => Some(Self::Additive),
            "mask" => Some(Self::Mask),
            "output" => Some(Self::Output),
            _ => None,
        }
    }
}

/// Direction of a typed graph pin.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AnimationGraphPinDirection {
    Input,
    Output,
}

/// Semantic domain transferred through one graph pin.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AnimationGraphPinValueKind {
    Pose,
    ScalarParameter,
    TargetMask,
}

/// One named graph pin and its cardinality contract.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct AnimationGraphPinDescriptor {
    id: &'static str,
    direction: AnimationGraphPinDirection,
    value_kind: AnimationGraphPinValueKind,
    required: bool,
    accepts_multiple: bool,
}

impl AnimationGraphPinDescriptor {
    pub const fn id(self) -> &'static str {
        self.id
    }

    pub const fn direction(self) -> AnimationGraphPinDirection {
        self.direction
    }

    pub const fn value_kind(self) -> AnimationGraphPinValueKind {
        self.value_kind
    }

    pub const fn required(self) -> bool {
        self.required
    }

    pub const fn accepts_multiple(self) -> bool {
        self.accepts_multiple
    }
}

/// Typed descriptor for a source graph node kind.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct AnimationGraphNodeDescriptor {
    kind: AnimationGraphNodeSchemaKind,
    owner: AnimationCompilerSchemaOwner,
    inputs: &'static [AnimationGraphPinDescriptor],
    outputs: &'static [AnimationGraphPinDescriptor],
}

impl AnimationGraphNodeDescriptor {
    pub const fn kind(self) -> AnimationGraphNodeSchemaKind {
        self.kind
    }

    pub const fn owner(self) -> AnimationCompilerSchemaOwner {
        self.owner
    }

    pub const fn inputs(self) -> &'static [AnimationGraphPinDescriptor] {
        self.inputs
    }

    pub const fn outputs(self) -> &'static [AnimationGraphPinDescriptor] {
        self.outputs
    }
}

/// State source kinds recognized by the current state-machine schema.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AnimationStateKindSchemaKind {
    Clip,
    BlendSpace1D,
    BlendSpace2D,
    SubMachine,
    GraphRef,
}

/// Immutable builtin compiler schema registry.
///
/// Dynamic plugin contributions will be admitted against the same owner/version and descriptor
/// contracts instead of bypassing source validation with untyped payloads.
pub struct AnimationCompilerSchemaRegistry;

impl AnimationCompilerSchemaRegistry {
    pub const BUILTIN_OWNER: AnimationCompilerSchemaOwner = AnimationCompilerSchemaOwner {
        id: "zircon.runtime.animation",
        version: AnimationCompilerSchemaVersion::new(1, 0),
    };

    pub const fn supported_asset_kinds() -> &'static [AnimationCompilerAssetKind] {
        &[
            AnimationCompilerAssetKind::Sequence,
            AnimationCompilerAssetKind::Graph,
            AnimationCompilerAssetKind::StateMachine,
        ]
    }

    pub const fn graph_node(
        kind: AnimationGraphNodeSchemaKind,
    ) -> &'static AnimationGraphNodeDescriptor {
        match kind {
            AnimationGraphNodeSchemaKind::Clip => &GRAPH_CLIP,
            AnimationGraphNodeSchemaKind::Blend => &GRAPH_BLEND,
            AnimationGraphNodeSchemaKind::Additive => &GRAPH_ADDITIVE,
            AnimationGraphNodeSchemaKind::Mask => &GRAPH_MASK,
            AnimationGraphNodeSchemaKind::Output => &GRAPH_OUTPUT,
        }
    }

    pub const fn supported_state_kinds() -> &'static [AnimationStateKindSchemaKind] {
        &[
            AnimationStateKindSchemaKind::Clip,
            AnimationStateKindSchemaKind::BlendSpace1D,
            AnimationStateKindSchemaKind::BlendSpace2D,
            AnimationStateKindSchemaKind::SubMachine,
            AnimationStateKindSchemaKind::GraphRef,
        ]
    }
}

const POSE_OUTPUT: AnimationGraphPinDescriptor = AnimationGraphPinDescriptor {
    id: "pose",
    direction: AnimationGraphPinDirection::Output,
    value_kind: AnimationGraphPinValueKind::Pose,
    required: true,
    accepts_multiple: false,
};
const POSE_INPUT: AnimationGraphPinDescriptor = AnimationGraphPinDescriptor {
    id: "pose",
    direction: AnimationGraphPinDirection::Input,
    value_kind: AnimationGraphPinValueKind::Pose,
    required: true,
    accepts_multiple: false,
};
const POSE_INPUTS: AnimationGraphPinDescriptor = AnimationGraphPinDescriptor {
    id: "inputs",
    direction: AnimationGraphPinDirection::Input,
    value_kind: AnimationGraphPinValueKind::Pose,
    required: true,
    accepts_multiple: true,
};
const BASE_INPUT: AnimationGraphPinDescriptor = AnimationGraphPinDescriptor {
    id: "base",
    direction: AnimationGraphPinDirection::Input,
    value_kind: AnimationGraphPinValueKind::Pose,
    required: true,
    accepts_multiple: false,
};
const ADDITIVE_INPUT: AnimationGraphPinDescriptor = AnimationGraphPinDescriptor {
    id: "additive",
    direction: AnimationGraphPinDirection::Input,
    value_kind: AnimationGraphPinValueKind::Pose,
    required: true,
    accepts_multiple: false,
};
const WEIGHT_PARAMETER_INPUT: AnimationGraphPinDescriptor = AnimationGraphPinDescriptor {
    id: "weight_parameter",
    direction: AnimationGraphPinDirection::Input,
    value_kind: AnimationGraphPinValueKind::ScalarParameter,
    required: false,
    accepts_multiple: false,
};
const TARGET_MASK_INPUT: AnimationGraphPinDescriptor = AnimationGraphPinDescriptor {
    id: "target_ids",
    direction: AnimationGraphPinDirection::Input,
    value_kind: AnimationGraphPinValueKind::TargetMask,
    required: false,
    accepts_multiple: true,
};

const GRAPH_CLIP_OUTPUTS: &[AnimationGraphPinDescriptor] = &[POSE_OUTPUT];
const GRAPH_BLEND_INPUTS: &[AnimationGraphPinDescriptor] = &[POSE_INPUTS, WEIGHT_PARAMETER_INPUT];
const GRAPH_ADDITIVE_INPUTS: &[AnimationGraphPinDescriptor] =
    &[BASE_INPUT, ADDITIVE_INPUT, WEIGHT_PARAMETER_INPUT];
const GRAPH_MASK_INPUTS: &[AnimationGraphPinDescriptor] = &[POSE_INPUT, TARGET_MASK_INPUT];
const GRAPH_OUTPUT_INPUTS: &[AnimationGraphPinDescriptor] = &[POSE_INPUT];

const GRAPH_CLIP: AnimationGraphNodeDescriptor = AnimationGraphNodeDescriptor {
    kind: AnimationGraphNodeSchemaKind::Clip,
    owner: AnimationCompilerSchemaRegistry::BUILTIN_OWNER,
    inputs: &[],
    outputs: GRAPH_CLIP_OUTPUTS,
};
const GRAPH_BLEND: AnimationGraphNodeDescriptor = AnimationGraphNodeDescriptor {
    kind: AnimationGraphNodeSchemaKind::Blend,
    owner: AnimationCompilerSchemaRegistry::BUILTIN_OWNER,
    inputs: GRAPH_BLEND_INPUTS,
    outputs: GRAPH_CLIP_OUTPUTS,
};
const GRAPH_ADDITIVE: AnimationGraphNodeDescriptor = AnimationGraphNodeDescriptor {
    kind: AnimationGraphNodeSchemaKind::Additive,
    owner: AnimationCompilerSchemaRegistry::BUILTIN_OWNER,
    inputs: GRAPH_ADDITIVE_INPUTS,
    outputs: GRAPH_CLIP_OUTPUTS,
};
const GRAPH_MASK: AnimationGraphNodeDescriptor = AnimationGraphNodeDescriptor {
    kind: AnimationGraphNodeSchemaKind::Mask,
    owner: AnimationCompilerSchemaRegistry::BUILTIN_OWNER,
    inputs: GRAPH_MASK_INPUTS,
    outputs: GRAPH_CLIP_OUTPUTS,
};
const GRAPH_OUTPUT: AnimationGraphNodeDescriptor = AnimationGraphNodeDescriptor {
    kind: AnimationGraphNodeSchemaKind::Output,
    owner: AnimationCompilerSchemaRegistry::BUILTIN_OWNER,
    inputs: GRAPH_OUTPUT_INPUTS,
    outputs: &[],
};
