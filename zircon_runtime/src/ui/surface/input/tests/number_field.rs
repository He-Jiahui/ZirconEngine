use super::{clamp, parse_number_field_value, snap_to_step, NumberFieldPolicy};
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
