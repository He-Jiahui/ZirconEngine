use super::*;
use crate::plugin::{PluginDistributionManifest, PluginPackageManifest};

#[test]
fn engine_compat_accepts_current_minor_range() {
    assert!(engine_compat_matches(">=0.1, <0.2", "0.1.0").unwrap());
}

#[test]
fn engine_compat_reports_empty_comparator_with_typed_error() {
    let error = engine_compat_matches(">=0.1, , <0.2", "0.1.0")
        .expect_err("empty comparator should be rejected");

    assert_eq!(error, NativeDistributionCompatibilityError::EmptyComparator);
    assert_eq!(error.to_string(), "empty comparator");
}

#[test]
fn engine_compat_reports_invalid_version_component_with_typed_error() {
    let error = engine_compat_matches(">=0.x", "0.1.0")
        .expect_err("invalid version component should be rejected");

    assert_eq!(
        error,
        NativeDistributionCompatibilityError::NonNumericVersionComponent {
            version: "0.x".to_string(),
            component: "x".to_string(),
        }
    );
    assert_eq!(
        error.to_string(),
        "version \"0.x\" contains non-numeric component \"x\""
    );
}

#[test]
fn distribution_diagnostic_rejects_unsupported_abi_version() {
    let manifest = PluginPackageManifest::new("future_native", "Future Native").with_distribution(
        PluginDistributionManifest {
            forms: vec!["dist".to_string()],
            abi_version: Some(ZIRCON_NATIVE_PLUGIN_ABI_VERSION_V3 + 1),
            engine_compat: ">=0.1, <0.2".to_string(),
            ..PluginDistributionManifest::default()
        },
    );

    let diagnostic = native_distribution_compatibility_diagnostic("future_native", &manifest)
        .expect("unsupported ABI should produce a diagnostic");

    assert!(diagnostic.contains("abi_version"));
    assert!(diagnostic.contains("incompatible with loader ABI"));
}
