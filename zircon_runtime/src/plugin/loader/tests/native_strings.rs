use super::*;

#[test]
fn read_required_c_string_reports_missing_field_with_typed_error() {
    let error = unsafe { read_required_c_string(std::ptr::null(), "plugin_id") }
        .expect_err("null required field should report typed string error");

    match error {
        NativeStringError::MissingRequiredField { field_name } => {
            assert_eq!(field_name, "plugin_id");
        }
        NativeStringError::InvalidPackageManifest { .. } => {
            panic!("null required field should not report package manifest parse error")
        }
    }
}

#[test]
fn native_string_typed_error_preserves_package_manifest_message() {
    let error = package_manifest_from_toml("not = [", "native plugin package manifest is invalid")
        .expect_err("invalid TOML should report typed package manifest error");

    assert!(
        error
            .to_string()
            .starts_with("native plugin package manifest is invalid: "),
        "typed package manifest error should preserve existing diagnostic prefix"
    );
    assert!(
        std::error::Error::source(&error).is_some(),
        "package manifest error should preserve TOML source"
    );
}
