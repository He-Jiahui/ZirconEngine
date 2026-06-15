use super::{PostProcessChainSlot, PostProcessEffectKind, PostProcessEffectSettings};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PostProcessPassNode {
    pub name: String,
    pub kind: PostProcessEffectKind,
    pub chain_slot: PostProcessChainSlot,
    pub planned_chain_executor_id: String,
    pub required_inputs: Vec<String>,
    pub produced_outputs: Vec<String>,
    pub after: Vec<PostProcessEffectKind>,
}

impl PostProcessPassNode {
    pub fn new(name: impl Into<String>, kind: PostProcessEffectKind) -> Self {
        let chain_slot = PostProcessChainSlot::from_current_effect_kind(kind);
        Self {
            name: name.into(),
            kind,
            chain_slot,
            planned_chain_executor_id: chain_slot.planned_executor_id().to_string(),
            required_inputs: Vec::new(),
            produced_outputs: Vec::new(),
            after: Vec::new(),
        }
    }

    pub fn from_settings(settings: &PostProcessEffectSettings) -> Self {
        Self::new(settings.kind.label(), settings.kind)
            .with_required_inputs(settings.required_inputs.clone())
            .with_produced_outputs(settings.produced_outputs.clone())
            .with_after(settings.after.clone())
    }

    pub fn with_required_inputs(
        mut self,
        resources: impl IntoIterator<Item = impl Into<String>>,
    ) -> Self {
        self.required_inputs = resources.into_iter().map(Into::into).collect();
        self
    }

    pub fn with_produced_outputs(
        mut self,
        resources: impl IntoIterator<Item = impl Into<String>>,
    ) -> Self {
        self.produced_outputs = resources.into_iter().map(Into::into).collect();
        self
    }

    pub fn with_after(
        mut self,
        dependencies: impl IntoIterator<Item = PostProcessEffectKind>,
    ) -> Self {
        self.after = dependencies.into_iter().collect();
        self
    }
}
