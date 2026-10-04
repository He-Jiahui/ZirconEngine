use std::ffi::CString;
use std::ptr;

use toml::Value;

use super::native_artifact::validate_entry_name;

#[test]
fn native_entry_name_validation_covers_absent_unexpected_missing_drift_and_match() {
    let empty_distribution = toml::map::Map::new();
    let mut diagnostics = Vec::new();
    assert_eq!(
        validate_entry_name(
            &empty_distribution,
            "editor_entry",
            ptr::null(),
            &mut diagnostics,
        ),
        None
    );
    assert!(diagnostics.is_empty());

    let unexpected = CString::new("unexpected_editor_entry").unwrap();
    assert_eq!(
        validate_entry_name(
            &empty_distribution,
            "editor_entry",
            unexpected.as_ptr(),
            &mut diagnostics,
        ),
        None
    );
    assert_eq!(
        diagnostics.last().unwrap().code,
        "plugin.native_artifact.editor_entry_unexpected"
    );

    let mut runtime_distribution = toml::map::Map::new();
    runtime_distribution.insert(
        "runtime_entry".to_string(),
        Value::String("expected_runtime_entry".to_string()),
    );
    assert_eq!(
        validate_entry_name(
            &runtime_distribution,
            "runtime_entry",
            ptr::null(),
            &mut diagnostics,
        ),
        None
    );
    assert_eq!(
        diagnostics.last().unwrap().code,
        "plugin.native_artifact.runtime_entry_null"
    );

    let drifted = CString::new("drifted_runtime_entry").unwrap();
    assert_eq!(
        validate_entry_name(
            &runtime_distribution,
            "runtime_entry",
            drifted.as_ptr(),
            &mut diagnostics,
        ),
        None
    );
    assert_eq!(
        diagnostics.last().unwrap().code,
        "plugin.native_artifact.runtime_entry_mismatch"
    );

    let expected = CString::new("expected_runtime_entry").unwrap();
    assert_eq!(
        validate_entry_name(
            &runtime_distribution,
            "runtime_entry",
            expected.as_ptr(),
            &mut diagnostics,
        ),
        Some("expected_runtime_entry".to_string())
    );
}
