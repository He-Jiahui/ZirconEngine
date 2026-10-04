//! 动态分发候选的加载前兼容门；加载和只验证路径共用此判断。
//! 逗号分隔的比较子句须同时成立，语法与构建时 plugin validate 对照。

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

/// 开库前检查 dist 形态、ABI 与当前引擎版本；有诊断时调用方跳过候选。
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

/// 按范围子句判断当前引擎是否可加载该插件；第一个不匹配即停止。
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
        // TODO: [CR-PLUGIN-NATIVE-0603] 确认短路后是否仍须校验尾部子句语法；
        // 当前前段不匹配时，后段畸形会被报告为版本不匹配而非无效范围。
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

/// 只比较数值 release 的 major/minor/patch；预发布及构建后缀不参与排序。
fn parse_engine_version(version: &str) -> NativeDistributionCompatibilityResult<EngineVersion> {
    let release = version
        .split(|ch| ch == '-' || ch == '+')
        .next()
        .unwrap_or_default()
        .trim();
    if release.is_empty() {
        return Err(NativeDistributionCompatibilityError::EmptyVersion);
    }
    let mut parts = release.split('.');
    let major = parts.next();
    let minor = parts.next();
    let patch = parts.next();
    let extra = parts.next();
    let (Some(major), Some(minor), patch, None) = (major, minor, patch, extra) else {
        return Err(NativeDistributionCompatibilityError::InvalidVersionShape {
            version: version.to_string(),
        });
    };
    let major = parse_version_component(major, version)?;
    let minor = parse_version_component(minor, version)?;
    let patch = patch
        .map(|component| parse_version_component(component, version))
        .transpose()?
        .unwrap_or_default();
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
#[path = "tests/compatibility.rs"]
mod tests;

#[cfg(test)]
#[path = "compatibility/tests/version_streaming_tests.rs"]
mod version_streaming_tests;
