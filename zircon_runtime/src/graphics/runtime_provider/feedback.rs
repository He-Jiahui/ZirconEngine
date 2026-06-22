#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct RuntimeProviderFeedback<G, V> {
    gpu_completion: Option<G>,
    visibility_feedback: Option<V>,
}

impl<G, V> Default for RuntimeProviderFeedback<G, V> {
    fn default() -> Self {
        Self {
            gpu_completion: None,
            visibility_feedback: None,
        }
    }
}

impl<G, V> RuntimeProviderFeedback<G, V> {
    pub(crate) fn new(gpu_completion: Option<G>, visibility_feedback: Option<V>) -> Self {
        Self {
            gpu_completion,
            visibility_feedback,
        }
    }

    pub(crate) fn gpu_completion(&self) -> Option<&G> {
        self.gpu_completion.as_ref()
    }

    pub(crate) fn visibility_feedback(&self) -> Option<&V> {
        self.visibility_feedback.as_ref()
    }
}
