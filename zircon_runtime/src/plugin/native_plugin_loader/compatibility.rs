use crate::plugin::PluginPackageManifest;

use super::ZIRCON_NATIVE_PLUGIN_ABI_VERSION_V3;

const CURRENT_ENGINE_VERSION: &str = env!("CARGO_PKG_VERSION");

type NativeDistributionCompatibilityResult<T> =
    std::result::Result<T, NativeDistributionCompatibilityError>;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
struct EngineVersion {
    major: u64,
    minor: u64,
    patch: u64,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum VersionComparator {
    GreaterThan,
    GreaterThanOrEqual,
    Equal,
    LessThan,
    LessThanOrEqual,
}

#[derive(Clone, Debug, PartialEq, Eq)]
enum NativeDistributionCompatibilityError {
    EmptyComparator,
    EmptyVersion,
    InvalidVersionShape { version: String },
    NonNumericVersionComponent { version: String, component: String },
}

impl std::fmt::Display for NativeDistributionCompatibilityError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::EmptyComparator => formatter.write_str("empty comparator"),
            Self::EmptyVersion => formatter.write_str("version is empty"),
            Self::InvalidVersionShape { version } => {
                write!(
                    formatter,
                    "version \"{version}\" must be major.minor[.patch]"
                )
            }
            Self::NonNumericVersionComponent { version, component } => write!(
                formatter,
                "version \"{version}\" contains non-numeric component \"{component}\""
            ),
        }
    }
}

pub(super) fn native_distribution_compatibility_diagnostic(
    plugin_id: &str,
    package_manifest: &PluginPackageManifest,
) -> Option<String> {
    let Some(distribution) = &package_manifest.distribution else {
        return None;
    };
    if !distribution.forms.iter().any(|form| form.trim() == "dist") {
        return Some(format!(
            "native plugin {plugin_id} skipped because distribution forms do not include dist"
        ));
    }
    match distribution.abi_version {
        Some(ZIRCON_NATIVE_PLUGIN_ABI_VERSION_V3) => {}
        Some(abi_version) => {
            return Some(format!(
                "native plugin {plugin_id} skipped because distribution abi_version {abi_version} is incompatible with loader ABI {}",
                ZIRCON_NATIVE_PLUGIN_ABI_VERSION_V3
            ));
        }
        None => {
            return Some(format!(
                "native plugin {plugin_id} skipped because distribution abi_version is missing; expected {}",
                ZIRCON_NATIVE_PLUGIN_ABI_VERSION_V3
            ));
        }
    }
    let engine_compat = distribution.engine_compat.trim();
    if engine_compat.is_empty() {
        return Some(format!(
            "native plugin {plugin_id} skipped because distribution engine_compat is missing"
        ));
    }
    match engine_compat_matches(engine_compat, CURRENT_ENGINE_VERSION) {
        Ok(true) => None,
        Ok(false) => Some(format!(
            "native plugin {plugin_id} skipped because distribution engine_compat \"{engine_compat}\" does not include engine {CURRENT_ENGINE_VERSION}"
        )),
        Err(error) => Some(format!(
            "native plugin {plugin_id} skipped because distribution engine_compat \"{engine_compat}\" is invalid: {error}"
        )),
    }
}

fn engine_compat_matches(
    range: &str,
    current: &str,
) -> NativeDistributionCompatibilityResult<bool> {
    let current = parse_engine_version(current)?;
    for clause in range.split(',') {
        let clause = clause.trim();
        if clause.is_empty() {
            return Err(NativeDistributionCompatibilityError::EmptyComparator);
        }
        let (comparator, version) = parse_comparator(clause)?;
        let matches = match comparator {
            VersionComparator::GreaterThan => current > version,
            VersionComparator::GreaterThanOrEqual => current >= version,
            VersionComparator::Equal => current == version,
            VersionComparator::LessThan => current < version,
            VersionComparator::LessThanOrEqual => current <= version,
        };
        if !matches {
            return Ok(false);
        }
    }
    Ok(true)
}

fn parse_comparator(
    clause: &str,
) -> NativeDistributionCompatibilityResult<(VersionComparator, EngineVersion)> {
    let (comparator, version) = if let Some(version) = clause.strip_prefix(">=") {
        (VersionComparator::GreaterThanOrEqual, version)
    } else if let Some(version) = clause.strip_prefix("<=") {
        (VersionComparator::LessThanOrEqual, version)
    } else if let Some(version) = clause.strip_prefix('>') {
        (VersionComparator::GreaterThan, version)
    } else if let Some(version) = clause.strip_prefix('<') {
        (VersionComparator::LessThan, version)
    } else if let Some(version) = clause.strip_prefix('=') {
        (VersionComparator::Equal, version)
    } else {
        (VersionComparator::Equal, clause)
    };
    Ok((comparator, parse_engine_version(version.trim())?))
}

fn parse_engine_version(version: &str) -> NativeDistributionCompatibilityResult<EngineVersion> {
    let release = version
        .split(|ch| ch == '-' || ch == '+')
        .next()
        .unwrap_or_default()
        .trim();
    if release.is_empty() {
        return Err(NativeDistributionCompatibilityError::EmptyVersion);
    }
    let parts = release.split('.').collect::<Vec<_>>();
    if parts.len() < 2 || parts.len() > 3 {
        return Err(NativeDistributionCompatibilityError::InvalidVersionShape {
            version: version.to_string(),
        });
    }
    let major = parse_version_component(parts[0], version)?;
    let minor = parse_version_component(parts[1], version)?;
    let patch = if parts.len() == 3 {
        parse_version_component(parts[2], version)?
    } else {
        0
    };
    Ok(EngineVersion {
        major,
        minor,
        patch,
    })
}

fn parse_version_component(
    component: &str,
    version: &str,
) -> NativeDistributionCompatibilityResult<u64> {
    component.parse::<u64>().map_err(|_| {
        NativeDistributionCompatibilityError::NonNumericVersionComponent {
            version: version.to_string(),
            component: component.to_string(),
        }
    })
}

#[cfg(test)]
mod tests {
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
        let manifest = PluginPackageManifest::new("future_native", "Future Native")
            .with_distribution(PluginDistributionManifest {
                forms: vec!["dist".to_string()],
                abi_version: Some(ZIRCON_NATIVE_PLUGIN_ABI_VERSION_V3 + 1),
                engine_compat: ">=0.1, <0.2".to_string(),
                ..PluginDistributionManifest::default()
            });

        let diagnostic = native_distribution_compatibility_diagnostic("future_native", &manifest)
            .expect("unsupported ABI should produce a diagnostic");

        assert!(diagnostic.contains("abi_version"));
        assert!(diagnostic.contains("incompatible with loader ABI"));
    }
}
