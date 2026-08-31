use zircon_runtime_interface::ui::{
    component::UiValue,
    dispatch::{
        UiNumberFormatIdentityV1, UiNumberInputCommitMethod, UiNumberInputCommitStatus,
        UiNumberInputParseStatus, UiNumberInputReceiptV1,
    },
    event_ui::UiNodeId,
    tree::UiTemplateNodeMetadata,
};

use super::super::surface::UiSurface;

const NUMBER_FIELD_PROFILE_COUNTER_NAMES: [&str; 8] = [
    "number_field_parse_count",
    "number_field_parse_input_bytes",
    "number_field_edit_decision_count",
    "number_field_typed_publish_count",
    "number_field_commit_decision_count",
    "number_field_clamped_commit_count",
    "number_field_snapped_commit_count",
    "number_field_keyboard_step_count",
];
pub(super) const MVP_MAX_NUMBER_FIELD_EDIT_BYTES: usize = 128;
pub(super) const NUMBER_FIELD_VALUE_REVISION_PROPERTY: &str = "number_value_revision";
pub(super) const NUMBER_FIELD_EDIT_BASE_REVISION_PROPERTY: &str = "number_edit_base_revision";

#[derive(Clone, Copy, Debug, PartialEq)]
pub(super) struct NumberFieldPolicy {
    pub(super) min: Option<f64>,
    pub(super) max: Option<f64>,
    pub(super) step: Option<f64>,
    pub(super) snap_on_commit: bool,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(super) struct ParsedNumberFieldValue {
    pub(super) status: UiNumberInputParseStatus,
    pub(super) value: Option<f64>,
}

#[derive(Clone, Debug, PartialEq)]
pub(in crate::ui) struct NumberFieldCommitDecision {
    pub(in crate::ui) receipt: UiNumberInputReceiptV1,
    pub(in crate::ui) value: f64,
    pub(in crate::ui) text: String,
    pub(in crate::ui) edit_active: bool,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(super) struct NumberFieldEditDecision {
    pub(super) receipt: UiNumberInputReceiptV1,
    pub(super) publish_value: Option<f64>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct NumberFieldRevisionProjection {
    pub(super) value_revision: i64,
    pub(super) edit_base_revision: i64,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(in crate::ui) enum NumberFieldRevisionError {
    InvalidState,
    Exhausted,
}

pub(super) fn number_field_edit_decision(
    surface: &UiSurface,
    target: UiNodeId,
    text: &str,
) -> Option<NumberFieldEditDecision> {
    crate::profile_counter!("runtime", NUMBER_FIELD_PROFILE_COUNTER_NAMES[2], 1);
    let metadata = number_field_metadata(surface, target)?;
    let parsed = parse_number_field_value(text, policy_from_metadata(metadata));
    let publish_value = metadata
        .attributes
        .get("number_publish_per_key")
        .and_then(toml::Value::as_bool)
        .unwrap_or(false)
        .then_some(parsed)
        .filter(|_| !number_field_edit_revision_is_stale(metadata))
        .filter(|parsed| parsed.status == UiNumberInputParseStatus::Valid)
        .and_then(|parsed| parsed.value);
    if publish_value.is_some() {
        crate::profile_counter!("runtime", NUMBER_FIELD_PROFILE_COUNTER_NAMES[3], 1);
    }
    Some(NumberFieldEditDecision {
        receipt: UiNumberInputReceiptV1 {
            format: UiNumberFormatIdentityV1::InvariantAscii,
            parse_status: parsed.status,
            ..UiNumberInputReceiptV1::default()
        },
        publish_value,
    })
}

pub(in crate::ui) fn number_field_edit_is_active(surface: &UiSurface, target: UiNodeId) -> bool {
    number_field_metadata(surface, target)
        .and_then(|metadata| metadata.attributes.get("number_edit_active"))
        .and_then(toml::Value::as_bool)
        .unwrap_or(false)
}

pub(in crate::ui) fn number_field_commit_decision(
    surface: &UiSurface,
    target: UiNodeId,
    text: &str,
    method: UiNumberInputCommitMethod,
) -> Option<NumberFieldCommitDecision> {
    crate::profile_counter!("runtime", NUMBER_FIELD_PROFILE_COUNTER_NAMES[4], 1);
    let metadata = number_field_metadata(surface, target)?;
    let current = canonical_value_from_metadata(metadata)?;
    let policy = policy_from_metadata(metadata);
    if method == UiNumberInputCommitMethod::Escape {
        return Some(NumberFieldCommitDecision {
            receipt: UiNumberInputReceiptV1 {
                parse_status: parse_number_field_value(text, policy).status,
                commit_method: method,
                commit_status: UiNumberInputCommitStatus::Cancelled,
                ..UiNumberInputReceiptV1::default()
            },
            value: current,
            text: UiValue::Float(current).display_text(),
            edit_active: false,
        });
    }

    if matches!(
        method,
        UiNumberInputCommitMethod::Enter | UiNumberInputCommitMethod::FocusLoss
    ) && number_field_edit_revision_is_stale(metadata)
    {
        let keep_editing = method == UiNumberInputCommitMethod::Enter;
        return Some(NumberFieldCommitDecision {
            receipt: UiNumberInputReceiptV1 {
                parse_status: parse_number_field_value(text, policy).status,
                commit_method: method,
                commit_status: UiNumberInputCommitStatus::Conflict,
                ..UiNumberInputReceiptV1::default()
            },
            value: current,
            text: if keep_editing {
                text.to_string()
            } else {
                UiValue::Float(current).display_text()
            },
            edit_active: keep_editing,
        });
    }

    let parsed = parse_number_field_value(text, policy);
    let Some(mut value) = parsed.value else {
        let keep_editing = method == UiNumberInputCommitMethod::Enter;
        return Some(NumberFieldCommitDecision {
            receipt: UiNumberInputReceiptV1 {
                parse_status: parsed.status,
                commit_method: method,
                commit_status: UiNumberInputCommitStatus::Rejected,
                ..UiNumberInputReceiptV1::default()
            },
            value: current,
            text: if keep_editing {
                text.to_string()
            } else {
                UiValue::Float(current).display_text()
            },
            edit_active: keep_editing,
        });
    };

    let before_policy = value;
    value = clamp(value, policy.min, policy.max);
    let clamped = value != before_policy;
    let mut snapped = false;
    if policy.snap_on_commit {
        let Some(snapped_value) = snap_to_step(value, policy.min, policy.step) else {
            return Some(NumberFieldCommitDecision {
                receipt: UiNumberInputReceiptV1 {
                    parse_status: UiNumberInputParseStatus::InvalidPolicy,
                    commit_method: method,
                    commit_status: UiNumberInputCommitStatus::Rejected,
                    ..UiNumberInputReceiptV1::default()
                },
                value: current,
                text: if method == UiNumberInputCommitMethod::Enter {
                    text.to_string()
                } else {
                    UiValue::Float(current).display_text()
                },
                edit_active: method == UiNumberInputCommitMethod::Enter,
            });
        };
        snapped = snapped_value != value;
        value = clamp(snapped_value, policy.min, policy.max);
    }
    let commit_status = if snapped {
        crate::profile_counter!("runtime", NUMBER_FIELD_PROFILE_COUNTER_NAMES[6], 1);
        UiNumberInputCommitStatus::Snapped
    } else if clamped {
        crate::profile_counter!("runtime", NUMBER_FIELD_PROFILE_COUNTER_NAMES[5], 1);
        UiNumberInputCommitStatus::Clamped
    } else if value == current {
        UiNumberInputCommitStatus::Unchanged
    } else {
        UiNumberInputCommitStatus::Applied
    };
    Some(NumberFieldCommitDecision {
        receipt: UiNumberInputReceiptV1 {
            parse_status: parsed.status,
            commit_method: method,
            commit_status,
            ..UiNumberInputReceiptV1::default()
        },
        value,
        text: UiValue::Float(value).display_text(),
        edit_active: false,
    })
}

pub(in crate::ui::surface::input) fn number_field_keyboard_step_decision(
    surface: &UiSurface,
    target: UiNodeId,
    direction: f64,
) -> Option<NumberFieldCommitDecision> {
    let metadata = number_field_metadata(surface, target)?;
    crate::profile_counter!("runtime", NUMBER_FIELD_PROFILE_COUNTER_NAMES[7], 1);
    let current = canonical_value_from_metadata(metadata)?;
    let policy = policy_from_metadata(metadata);
    let Some(step) = policy.step.filter(|step| step.is_finite() && *step > 0.0) else {
        return Some(rejected_keyboard_step(
            metadata,
            current,
            UiNumberInputParseStatus::InvalidPolicy,
        ));
    };
    if !policy_is_valid(policy) || !matches!(direction, -1.0 | 1.0) {
        return Some(rejected_keyboard_step(
            metadata,
            current,
            UiNumberInputParseStatus::InvalidPolicy,
        ));
    }
    let candidate = current + direction * step;
    if !candidate.is_finite() {
        return Some(rejected_keyboard_step(
            metadata,
            current,
            UiNumberInputParseStatus::NonFinite,
        ));
    }
    let value = clamp(candidate, policy.min, policy.max);
    let clamped = value != candidate;
    let commit_status = if clamped {
        crate::profile_counter!("runtime", NUMBER_FIELD_PROFILE_COUNTER_NAMES[5], 1);
        UiNumberInputCommitStatus::Clamped
    } else if value == current {
        UiNumberInputCommitStatus::Unchanged
    } else {
        UiNumberInputCommitStatus::Applied
    };
    Some(NumberFieldCommitDecision {
        receipt: UiNumberInputReceiptV1 {
            parse_status: if clamped {
                UiNumberInputParseStatus::OutOfRange
            } else {
                UiNumberInputParseStatus::Valid
            },
            commit_method: UiNumberInputCommitMethod::KeyboardStep,
            commit_status,
            ..UiNumberInputReceiptV1::default()
        },
        value,
        text: UiValue::Float(value).display_text(),
        edit_active: false,
    })
}

fn rejected_keyboard_step(
    metadata: &UiTemplateNodeMetadata,
    current: f64,
    parse_status: UiNumberInputParseStatus,
) -> NumberFieldCommitDecision {
    let text = metadata
        .attributes
        .get("value_text")
        .and_then(toml::Value::as_str)
        .map(str::to_string)
        .unwrap_or_else(|| UiValue::Float(current).display_text());
    let edit_active = metadata
        .attributes
        .get("number_edit_active")
        .and_then(toml::Value::as_bool)
        .unwrap_or(false);
    NumberFieldCommitDecision {
        receipt: UiNumberInputReceiptV1 {
            parse_status,
            commit_method: UiNumberInputCommitMethod::KeyboardStep,
            commit_status: UiNumberInputCommitStatus::Rejected,
            ..UiNumberInputReceiptV1::default()
        },
        value: current,
        text,
        edit_active,
    }
}

pub(super) fn number_field_revision_projection(
    surface: &UiSurface,
    target: UiNodeId,
    next_value: &UiValue,
    edit_active: bool,
    preserve_edit_base: bool,
) -> Result<NumberFieldRevisionProjection, NumberFieldRevisionError> {
    let metadata =
        number_field_metadata(surface, target).ok_or(NumberFieldRevisionError::InvalidState)?;
    let current_value =
        canonical_value_from_metadata(metadata).ok_or(NumberFieldRevisionError::InvalidState)?;
    let UiValue::Float(next_value) = next_value else {
        return Err(NumberFieldRevisionError::InvalidState);
    };
    if !next_value.is_finite() {
        return Err(NumberFieldRevisionError::InvalidState);
    }
    let (value_revision, edit_base_revision, current_edit_active) =
        revision_state_from_metadata(metadata)?;
    let value_changed = current_value != *next_value;
    let next_value_revision = if value_changed {
        value_revision
            .checked_add(1)
            .ok_or(NumberFieldRevisionError::Exhausted)?
    } else {
        value_revision
    };
    let next_edit_base_revision = if preserve_edit_base && current_edit_active {
        edit_base_revision
    } else if edit_active {
        if value_changed {
            next_value_revision
        } else if current_edit_active {
            edit_base_revision
        } else {
            value_revision
        }
    } else {
        next_value_revision
    };
    Ok(NumberFieldRevisionProjection {
        value_revision: next_value_revision,
        edit_base_revision: next_edit_base_revision,
    })
}

fn revision_attribute(
    metadata: &UiTemplateNodeMetadata,
    property: &str,
) -> Result<i64, NumberFieldRevisionError> {
    match metadata.attributes.get(property) {
        None => Ok(0),
        Some(toml::Value::Integer(value)) if *value >= 0 => Ok(*value),
        Some(_) => Err(NumberFieldRevisionError::InvalidState),
    }
}

fn edit_active_attribute(
    metadata: &UiTemplateNodeMetadata,
) -> Result<bool, NumberFieldRevisionError> {
    match metadata.attributes.get("number_edit_active") {
        None => Ok(false),
        Some(toml::Value::Boolean(value)) => Ok(*value),
        Some(_) => Err(NumberFieldRevisionError::InvalidState),
    }
}

fn revision_state_from_metadata(
    metadata: &UiTemplateNodeMetadata,
) -> Result<(i64, i64, bool), NumberFieldRevisionError> {
    let value_revision = revision_attribute(metadata, NUMBER_FIELD_VALUE_REVISION_PROPERTY)?;
    let edit_base_revision =
        revision_attribute(metadata, NUMBER_FIELD_EDIT_BASE_REVISION_PROPERTY)?;
    let edit_active = edit_active_attribute(metadata)?;
    if edit_base_revision > value_revision || (!edit_active && edit_base_revision != value_revision)
    {
        return Err(NumberFieldRevisionError::InvalidState);
    }
    Ok((value_revision, edit_base_revision, edit_active))
}

pub(in crate::ui) fn number_field_value_revision(
    surface: &UiSurface,
    target: UiNodeId,
) -> Result<i64, NumberFieldRevisionError> {
    let metadata =
        number_field_metadata(surface, target).ok_or(NumberFieldRevisionError::InvalidState)?;
    canonical_value_from_metadata(metadata).ok_or(NumberFieldRevisionError::InvalidState)?;
    revision_state_from_metadata(metadata).map(|(value_revision, _, _)| value_revision)
}

fn number_field_edit_revision_is_stale(metadata: &UiTemplateNodeMetadata) -> bool {
    match revision_state_from_metadata(metadata) {
        Ok((value_revision, edit_base_revision, edit_active)) => {
            edit_active && edit_base_revision != value_revision
        }
        Err(_) => true,
    }
}

pub(super) fn parse_number_field_value(
    text: &str,
    policy: NumberFieldPolicy,
) -> ParsedNumberFieldValue {
    crate::profile_scope!("runtime", "ui_text.number", "parse");
    crate::profile_counter!("runtime", NUMBER_FIELD_PROFILE_COUNTER_NAMES[0], 1);
    crate::profile_counter!("runtime", NUMBER_FIELD_PROFILE_COUNTER_NAMES[1], text.len());
    if !policy_is_valid(policy) {
        return parsed(UiNumberInputParseStatus::InvalidPolicy, None);
    }
    if text.len() > MVP_MAX_NUMBER_FIELD_EDIT_BYTES {
        return parsed(UiNumberInputParseStatus::TooLong, None);
    }
    if text.is_empty() {
        return parsed(UiNumberInputParseStatus::Empty, None);
    }
    if !text
        .bytes()
        .all(|byte| byte.is_ascii_digit() || matches!(byte, b'+' | b'-' | b'.' | b'e' | b'E'))
    {
        return parsed(UiNumberInputParseStatus::InvalidCharacter, None);
    }
    if is_intermediate_number(text) {
        return parsed(UiNumberInputParseStatus::Intermediate, None);
    }
    let Ok(value) = text.parse::<f64>() else {
        return parsed(UiNumberInputParseStatus::InvalidSyntax, None);
    };
    if !value.is_finite() {
        return parsed(UiNumberInputParseStatus::NonFinite, None);
    }
    let status = if policy.min.is_some_and(|minimum| value < minimum)
        || policy.max.is_some_and(|maximum| value > maximum)
    {
        UiNumberInputParseStatus::OutOfRange
    } else {
        UiNumberInputParseStatus::Valid
    };
    parsed(status, Some(value))
}

fn policy_is_valid(policy: NumberFieldPolicy) -> bool {
    policy.min.is_none_or(f64::is_finite)
        && policy.max.is_none_or(f64::is_finite)
        && !matches!((policy.min, policy.max), (Some(min), Some(max)) if min > max)
        && policy
            .step
            .is_none_or(|step| step.is_finite() && step >= 0.0)
        && (!policy.snap_on_commit || policy.step.is_some_and(|step| step > 0.0))
}

fn number_field_metadata(surface: &UiSurface, target: UiNodeId) -> Option<&UiTemplateNodeMetadata> {
    surface
        .tree
        .node(target)
        .and_then(|node| node.template_metadata.as_ref())
        .filter(|metadata| metadata.component == "NumberField")
}

fn canonical_value_from_metadata(metadata: &UiTemplateNodeMetadata) -> Option<f64> {
    metadata
        .attributes
        .get(metadata.widget.value_property.as_deref().unwrap_or("value"))
        .and_then(toml::Value::as_float)
        .filter(|value| value.is_finite())
}

fn policy_from_metadata(metadata: &UiTemplateNodeMetadata) -> NumberFieldPolicy {
    let format_is_invariant = metadata
        .attributes
        .get("number_format")
        .is_none_or(|value| value.as_str() == Some("invariant_ascii"));
    let snap_value = metadata.attributes.get("number_snap_on_commit");
    let publish_value = metadata.attributes.get("number_publish_per_key");
    let snap_on_commit = snap_value.and_then(toml::Value::as_bool).unwrap_or(false);
    let mut policy = NumberFieldPolicy {
        min: policy_number(metadata, "min"),
        max: policy_number(metadata, "max"),
        step: policy_number(metadata, "step"),
        snap_on_commit,
    };
    if !format_is_invariant
        || snap_value.is_some_and(|value| value.as_bool().is_none())
        || publish_value.is_some_and(|value| value.as_bool().is_none())
    {
        policy.min = Some(f64::NAN);
    }
    policy
}

fn value_as_f64(value: &toml::Value) -> Option<f64> {
    match value {
        toml::Value::Float(value) => Some(*value),
        toml::Value::Integer(value) => Some(*value as f64),
        _ => None,
    }
}

fn policy_number(metadata: &UiTemplateNodeMetadata, key: &str) -> Option<f64> {
    metadata
        .attributes
        .get(key)
        .map(|value| value_as_f64(value).unwrap_or(f64::NAN))
}

fn clamp(value: f64, min: Option<f64>, max: Option<f64>) -> f64 {
    value
        .max(min.unwrap_or(f64::NEG_INFINITY))
        .min(max.unwrap_or(f64::INFINITY))
}

fn snap_to_step(value: f64, min: Option<f64>, step: Option<f64>) -> Option<f64> {
    let step = step.filter(|step| step.is_finite() && *step > 0.0)?;
    let origin = min.unwrap_or(0.0);
    let delta = value - origin;
    if !delta.is_finite() {
        return None;
    }
    let quotient = delta / step;
    if !quotient.is_finite() {
        return None;
    }
    let snapped = origin + quotient.round() * step;
    snapped.is_finite().then_some(snapped)
}

fn is_intermediate_number(text: &str) -> bool {
    if matches!(text, "+" | "-" | "." | "+." | "-.") {
        return true;
    }
    let Some((mantissa, exponent)) = text.split_once(['e', 'E']) else {
        return false;
    };
    !mantissa.is_empty() && mantissa.parse::<f64>().is_ok() && matches!(exponent, "" | "+" | "-")
}

const fn parsed(status: UiNumberInputParseStatus, value: Option<f64>) -> ParsedNumberFieldValue {
    ParsedNumberFieldValue { status, value }
}

#[cfg(test)]
mod tests {
    use super::{NumberFieldPolicy, clamp, parse_number_field_value, snap_to_step};
    use zircon_runtime_interface::ui::dispatch::UiNumberInputParseStatus;

    const POLICY: NumberFieldPolicy = NumberFieldPolicy {
        min: Some(0.0),
        max: Some(100.0),
        step: Some(1.0),
        snap_on_commit: false,
    };

    #[test]
    fn invariant_parser_distinguishes_intermediate_valid_and_out_of_range_text() {
        for text in ["", "-", ".", "1e", "1e-", "+."] {
            let expected = if text.is_empty() {
                UiNumberInputParseStatus::Empty
            } else {
                UiNumberInputParseStatus::Intermediate
            };
            assert_eq!(parse_number_field_value(text, POLICY).status, expected);
        }
        assert_eq!(
            parse_number_field_value("12.5e1", POLICY).status,
            UiNumberInputParseStatus::OutOfRange
        );
        assert_eq!(
            parse_number_field_value("12.5", POLICY).status,
            UiNumberInputParseStatus::Valid
        );
    }

    #[test]
    fn invariant_parser_rejects_non_finite_invalid_character_and_invalid_policy() {
        assert_eq!(
            parse_number_field_value("1e999", POLICY).status,
            UiNumberInputParseStatus::NonFinite
        );
        assert_eq!(
            parse_number_field_value("12,5", POLICY).status,
            UiNumberInputParseStatus::InvalidCharacter
        );
        assert_eq!(
            parse_number_field_value(
                &"1".repeat(super::MVP_MAX_NUMBER_FIELD_EDIT_BYTES + 1),
                POLICY,
            )
            .status,
            UiNumberInputParseStatus::TooLong
        );
        assert_eq!(
            parse_number_field_value(
                "12",
                NumberFieldPolicy {
                    min: Some(10.0),
                    max: Some(1.0),
                    ..POLICY
                },
            )
            .status,
            UiNumberInputParseStatus::InvalidPolicy
        );
    }

    #[test]
    fn numeric_policy_clamps_without_depending_on_locale_or_source_text() {
        assert_eq!(clamp(-1.0, Some(0.0), Some(100.0)), 0.0);
        assert_eq!(clamp(101.0, Some(0.0), Some(100.0)), 100.0);
        assert_eq!(clamp(12.5, Some(0.0), Some(100.0)), 12.5);
    }

    #[test]
    fn numeric_step_snap_rejects_missing_zero_and_non_finite_arithmetic() {
        assert_eq!(snap_to_step(12.4, Some(0.0), Some(1.0)), Some(12.0));
        assert_eq!(snap_to_step(12.4, Some(0.0), None), None);
        assert_eq!(snap_to_step(12.4, Some(0.0), Some(0.0)), None);
        assert_eq!(snap_to_step(f64::MAX, Some(-f64::MAX), Some(1.0)), None);
    }
}
