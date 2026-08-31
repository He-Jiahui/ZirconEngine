use crate::ui::binding::{EditorUiBinding, EditorUiBindingPayload, EditorUiEventKind};
use zircon_runtime_interface::ui::component::{
    UiComponentAdapterError, UiComponentAdapterResult, UiComponentEvent, UiComponentEventEnvelope,
    UiValue, UiValueKind,
};

pub(crate) const COMMAND_DOMAIN: &str = "command";
const COMMITTED_COMMAND_ID: &str = "committed_command_id";
const SELECTED_COMMAND_ID: &str = "selected_command_id";
const COMMAND_ID: &str = "command_id";

pub(crate) fn editor_command_binding_for_envelope(
    envelope: &UiComponentEventEnvelope,
) -> Result<EditorUiBinding, UiComponentAdapterError> {
    if envelope.target.domain != COMMAND_DOMAIN {
        return Err(UiComponentAdapterError::UnsupportedTargetDomain {
            domain: envelope.target.domain.clone(),
        });
    }

    let command_id = command_id_from_event(envelope)?;
    Ok(EditorUiBinding::new(
        &envelope.document_id,
        &envelope.control_id,
        EditorUiEventKind::Submit,
        EditorUiBindingPayload::editor_command(command_id),
    ))
}

pub(crate) fn command_adapter_result(command_id: &str) -> UiComponentAdapterResult {
    UiComponentAdapterResult::changed()
        .dirty(false)
        .with_mutation_source(COMMAND_DOMAIN)
        .with_transaction(command_transaction(command_id))
        .with_status(command_status(command_id))
}

fn command_transaction(command_id: &str) -> String {
    const PREFIX: &str = "command:";
    let mut transaction = String::with_capacity(PREFIX.len() + command_id.len());
    transaction.push_str(PREFIX);
    transaction.push_str(command_id);
    transaction
}

fn command_status(command_id: &str) -> String {
    const PREFIX: &str = "Executed editor command `";
    const SUFFIX: &str = "`";
    let mut status = String::with_capacity(PREFIX.len() + command_id.len() + SUFFIX.len());
    status.push_str(PREFIX);
    status.push_str(command_id);
    status.push_str(SUFFIX);
    status
}

fn command_id_from_event(
    envelope: &UiComponentEventEnvelope,
) -> Result<String, UiComponentAdapterError> {
    match &envelope.event {
        UiComponentEvent::Commit { property, value } => {
            validate_command_property(envelope, property)?;
            let command_id =
                string_value(value).ok_or_else(|| UiComponentAdapterError::InvalidValueKind {
                    domain: envelope.target.domain.clone(),
                    path: envelope.target.path.clone(),
                    expected: UiValueKind::String,
                    actual: value.kind(),
                })?;
            if command_id.is_empty() {
                return Err(UiComponentAdapterError::RejectedInput {
                    domain: envelope.target.domain.clone(),
                    path: envelope.target.path.clone(),
                    reason: "command commit requires a non-empty command id".to_string(),
                });
            }
            Ok(command_id.to_string())
        }
        event => Err(UiComponentAdapterError::UnsupportedEvent {
            domain: envelope.target.domain.clone(),
            path: envelope.target.path.clone(),
            event_kind: event.kind(),
        }),
    }
}

fn validate_command_property(
    envelope: &UiComponentEventEnvelope,
    property: &str,
) -> Result<(), UiComponentAdapterError> {
    if matches!(
        property,
        COMMITTED_COMMAND_ID | SELECTED_COMMAND_ID | COMMAND_ID
    ) {
        return Ok(());
    }

    Err(UiComponentAdapterError::UnsupportedTargetPath {
        domain: envelope.target.domain.clone(),
        path: envelope.target.path.clone(),
    })
}

fn string_value(value: &UiValue) -> Option<&str> {
    match value {
        UiValue::String(value) => Some(value.trim()),
        _ => None,
    }
}

#[cfg(test)]
mod optimization_batch_fk_tests {
    use std::hint::black_box;
    use std::time::Instant;

    use super::*;

    const SAMPLE_PAIRS: usize = 17;
    const RESULTS_PER_SAMPLE: usize = 262_144;

    #[test]
    fn optimization_batch_fk_editor397_command_result_strings_preserve_bytes() {
        for command_id in ["save", "editor.scene.frame_selection", "plugin.command/42"] {
            assert_eq!(
                command_result_strings(command_id),
                legacy_command_result_strings(command_id)
            );
        }
    }

    #[test]
    #[ignore = "release performance gate"]
    fn optimization_batch_fk_editor397_direct_command_result_strings_benchmark() {
        const COMMAND_ID: &str = "editor.scene.frame_selected_entities";
        for _ in 0..4 {
            black_box(measure(legacy_command_result_strings, COMMAND_ID));
            black_box(measure(command_result_strings, COMMAND_ID));
        }
        let mut legacy_samples = Vec::with_capacity(SAMPLE_PAIRS);
        let mut optimized_samples = Vec::with_capacity(SAMPLE_PAIRS);
        for pair_index in 0..SAMPLE_PAIRS {
            if pair_index % 2 == 0 {
                legacy_samples.push(measure(legacy_command_result_strings, COMMAND_ID));
                optimized_samples.push(measure(command_result_strings, COMMAND_ID));
            } else {
                optimized_samples.push(measure(command_result_strings, COMMAND_ID));
                legacy_samples.push(measure(legacy_command_result_strings, COMMAND_ID));
            }
        }

        report_performance(&legacy_samples, &optimized_samples);
    }

    fn command_result_strings(command_id: &str) -> [String; 2] {
        [command_transaction(command_id), command_status(command_id)]
    }

    fn legacy_command_result_strings(command_id: &str) -> [String; 2] {
        [
            format!("command:{command_id}"),
            format!("Executed editor command `{command_id}`"),
        ]
    }

    fn measure(mut build: impl FnMut(&str) -> [String; 2], command_id: &str) -> u128 {
        let started = Instant::now();
        let mut checksum = 0_usize;
        for _ in 0..RESULTS_PER_SAMPLE {
            let strings = black_box(build(black_box(command_id)));
            checksum = checksum.wrapping_add(strings[0].len() + strings[1].len());
        }
        black_box(checksum);
        started.elapsed().as_nanos().max(1)
    }

    fn report_performance(legacy_samples: &[u128], optimized_samples: &[u128]) {
        let legacy_p95 = nearest_rank_p95(legacy_samples);
        let optimized_p95 = nearest_rank_p95(optimized_samples);
        let improvement_percent =
            legacy_p95.saturating_sub(optimized_p95).saturating_mul(100) / legacy_p95.max(1);
        println!(
            "EDITOR397_DIRECT_COMMAND_RESULT_STRINGS_BENCH_V1 sample_pairs={SAMPLE_PAIRS} results_per_sample={RESULTS_PER_SAMPLE} legacy_ns={} optimized_ns={} legacy_p95_ns={legacy_p95} optimized_p95_ns={optimized_p95} improvement_percent={improvement_percent} threshold_percent=25",
            csv(legacy_samples),
            csv(optimized_samples),
        );
        assert!(
            optimized_p95.saturating_mul(100) <= legacy_p95.saturating_mul(75),
            "optimized p95 {optimized_p95}ns must be at most 75% of legacy p95 {legacy_p95}ns"
        );
    }

    fn nearest_rank_p95(samples: &[u128]) -> u128 {
        let mut sorted = samples.to_vec();
        sorted.sort_unstable();
        sorted[(sorted.len() * 95).div_ceil(100).saturating_sub(1)]
    }

    fn csv(samples: &[u128]) -> String {
        samples
            .iter()
            .map(u128::to_string)
            .collect::<Vec<_>>()
            .join(",")
    }
}
