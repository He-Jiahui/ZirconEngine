#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum AnimationAuthoringDocumentKind {
    Sequence,
    Graph,
    StateMachine,
}

impl AnimationAuthoringDocumentKind {
    pub(crate) const fn label(self) -> &'static str {
        match self {
            Self::Sequence => "sequence",
            Self::Graph => "graph",
            Self::StateMachine => "state machine",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum AnimationGraphNodeKind {
    Output,
    Blend,
}
