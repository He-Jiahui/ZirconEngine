//! Stable diagnostics emitted while compiling animation authoring assets.

/// Severity of a semantic animation compilation diagnostic.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AnimationCompileSeverity {
    Error,
    Warning,
}

/// The authoring element that owns an animation compilation diagnostic.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum AnimationCompileElement {
    Asset,
    GraphNode(String),
    GraphOutput,
    GraphParameter(String),
    StateMachineState(String),
    StateMachineTransition {
        transition_index: usize,
        from_state: String,
        to_state: String,
    },
    StateMachineCondition {
        transition_index: usize,
        condition_index: usize,
        parameter: String,
    },
    StateMachineLayer(String),
    SequenceBinding {
        binding_index: usize,
    },
    SequenceTrack {
        binding_index: usize,
        track_index: usize,
        property_path: String,
    },
    SequenceKey {
        binding_index: usize,
        track_index: usize,
        key_index: usize,
    },
}

/// A deterministic, machine-readable animation compilation diagnostic.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AnimationCompileDiagnostic {
    code: &'static str,
    severity: AnimationCompileSeverity,
    element: AnimationCompileElement,
    message: String,
}

impl AnimationCompileDiagnostic {
    pub(crate) fn new(
        code: &'static str,
        severity: AnimationCompileSeverity,
        element: AnimationCompileElement,
        message: impl Into<String>,
    ) -> Self {
        Self {
            code,
            severity,
            element,
            message: message.into(),
        }
    }

    /// Stable diagnostic identifier suitable for editor presentation and automation.
    pub fn code(&self) -> &'static str {
        self.code
    }

    pub fn severity(&self) -> AnimationCompileSeverity {
        self.severity
    }

    pub fn element(&self) -> &AnimationCompileElement {
        &self.element
    }

    pub fn message(&self) -> &str {
        &self.message
    }
}
