use super::{PostProcessEffectKind, PostProcessEffectSettings};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PostProcessPassNode {
    pub name: String,
    pub kind: PostProcessEffectKind,
    pub required_inputs: Vec<String>,
    pub produced_outputs: Vec<String>,
    pub after: Vec<PostProcessEffectKind>,
}

impl PostProcessPassNode {
    pub fn from_settings(settings: &PostProcessEffectSettings) -> Self {
        Self {
            name: settings.kind.label().to_string(),
            kind: settings.kind,
            required_inputs: settings.required_inputs.clone(),
            produced_outputs: settings.produced_outputs.clone(),
            after: settings.after.clone(),
        }
    }
}
