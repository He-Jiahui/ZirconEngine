use zircon_runtime::core::framework::ai::{AiBehaviorNodeParameter, AiBehaviorNodeParameterValue};

/// Weight values compiled once with a random-selector node.
///
/// The executor deliberately keeps the existing floating-point/default-hash
/// selection contract in this slice. Only parameter resolution and temporary
/// weight-vector construction move out of the tick hot path.
#[derive(Clone, Debug, PartialEq)]
pub(super) struct CompiledRandomSelectorWeights {
    weights: Box<[f32]>,
    total: f32,
}

impl CompiledRandomSelectorWeights {
    pub(super) fn compile<'a>(
        parameters: &[AiBehaviorNodeParameter],
        child_ids: impl IntoIterator<Item = &'a str>,
    ) -> Self {
        let weights = child_ids
            .into_iter()
            .enumerate()
            .map(|(position, child_id)| resolve_weight(parameters, child_id, position))
            .collect::<Vec<_>>()
            .into_boxed_slice();
        let total = weights.iter().copied().sum();
        Self { weights, total }
    }

    pub(super) fn weights(&self) -> &[f32] {
        &self.weights
    }

    pub(super) const fn total(&self) -> f32 {
        self.total
    }
}

fn resolve_weight(parameters: &[AiBehaviorNodeParameter], child_id: &str, position: usize) -> f32 {
    let id_parameter = parameters
        .iter()
        .find(|parameter| parameter.key.strip_prefix("weight.") == Some(child_id));
    if let Some(AiBehaviorNodeParameterValue::Scalar(value)) =
        id_parameter.map(|parameter| &parameter.value)
    {
        return value.max(0.0);
    }

    let position_parameter = parameters.iter().find(|parameter| {
        parameter
            .key
            .strip_prefix("weight_")
            .is_some_and(|suffix| canonical_index_suffix_matches(suffix, position))
    });
    match position_parameter.map(|parameter| &parameter.value) {
        Some(AiBehaviorNodeParameterValue::Scalar(value)) => value.max(0.0),
        _ => 1.0,
    }
}

fn canonical_index_suffix_matches(suffix: &str, index: usize) -> bool {
    let bytes = suffix.as_bytes();
    if index == 0 {
        return bytes == b"0";
    }
    let mut remaining = index;
    let mut cursor = bytes.len();
    while remaining != 0 {
        if cursor == 0 {
            return false;
        }
        cursor -= 1;
        if bytes[cursor] != b'0' + (remaining % 10) as u8 {
            return false;
        }
        remaining /= 10;
    }
    cursor == 0
}
