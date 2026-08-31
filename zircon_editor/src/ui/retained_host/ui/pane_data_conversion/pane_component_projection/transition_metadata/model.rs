pub(in super::super) struct ProjectedTransitionMetadata {
    pub(in super::super) kind: String,
    pub(in super::super) active: bool,
    pub(in super::super) entered: bool,
    pub(in super::super) progress: f32,
    pub(in super::super) duration_ms: i32,
    pub(in super::super) easing: String,
    pub(in super::super) direction: String,
}

impl ProjectedTransitionMetadata {
    pub(super) fn without_transition(kind: String) -> Self {
        debug_assert!(kind.is_empty());
        Self {
            kind,
            active: true,
            entered: true,
            progress: 1.0,
            duration_ms: 0,
            easing: String::new(),
            direction: String::new(),
        }
    }
}
