use std::fmt;
use std::path::{Component, Path, PathBuf};

use serde::de::{DeserializeSeed, Error as _, IgnoredAny, MapAccess, SeqAccess, Visitor};

use super::candidate_from_manifest::{
    metered_candidate_from_manifest_path, read_bounded_utf8_file, resolve_manifest_path,
};
use super::discover::authority::{discovery_authority, input_identity};
use super::discovery_refresh::{
    NativePluginDiscoveryRefreshError, NativePluginDiscoveryRefreshRequest,
    NativePluginDiscoveryRefreshScratchReservation, NativePluginDiscoveryRefreshSink,
};
use super::{
    NativePluginCandidate, NativePluginLoadManifest, NativePluginLoadManifestEntry,
    NativePluginLoadReport, NativePluginLoader,
};

const NATIVE_PLUGIN_LOAD_MANIFEST_PATH: &str = "plugins/native_plugins.toml";

impl NativePluginLoader {
    pub fn discover_from_load_manifest(
        &self,
        export_root: impl AsRef<Path>,
    ) -> NativePluginLoadReport {
        discovery_authority().discover_load_manifest(export_root.as_ref())
    }

    pub fn load_all_from_load_manifest(
        &self,
        export_root: impl AsRef<Path>,
    ) -> NativePluginLoadReport {
        let report = self.discover_from_load_manifest(export_root);
        self.load_all_candidates(report)
    }

    pub fn load_runtime_from_load_manifest(
        &self,
        export_root: impl AsRef<Path>,
    ) -> NativePluginLoadReport {
        let report = self.discover_from_load_manifest(export_root);
        self.load_candidates_for_module_kinds(report, &[crate::plugin::PluginModuleKind::Runtime])
    }

    pub fn load_editor_from_load_manifest(
        &self,
        export_root: impl AsRef<Path>,
    ) -> NativePluginLoadReport {
        let report = self.discover_from_load_manifest(export_root);
        self.load_candidates_for_module_kinds(report, &[crate::plugin::PluginModuleKind::Editor])
    }
}

pub(in crate::plugin::native_plugin_loader) fn native_plugin_load_manifest_path(
    export_root: &Path,
) -> PathBuf {
    export_root.join(NATIVE_PLUGIN_LOAD_MANIFEST_PATH)
}

/// The authority is the only caller. This retains explicit export selection while flowing every
/// read, parse, candidate, diagnostic, ticket, and publication through one refresh generation.
pub(in crate::plugin::native_plugin_loader) fn collect_load_manifest(
    request: &NativePluginDiscoveryRefreshRequest,
    sink: &mut NativePluginDiscoveryRefreshSink,
    export_root: &Path,
) -> Result<
    super::discovery_refresh::NativePluginDiscoveryInputIdentity,
    NativePluginDiscoveryRefreshError,
> {
    request.check_active()?;
    let load_manifest_path = native_plugin_load_manifest_path(export_root);
    let source = match read_bounded_utf8_file(
        request,
        sink,
        &load_manifest_path,
        "native plugin load manifest",
    ) {
        Ok(source) => source,
        Err(error) => return emit_recoverable_error(request, sink, error, 0),
    };
    let parse_admission = reserve_load_manifest_parse_scratch(request, sink, source.len() as u64)?;
    let load_manifest = match parse_bounded_load_manifest(&source, request.budget().max_candidates)
    {
        Ok(load_manifest) => load_manifest,
        Err(error) => {
            emit_diagnostic(request, sink, || {
                format!(
                    "failed to parse native plugin load manifest {}: {error}",
                    load_manifest_path.display()
                )
            })?;
            return input_identity(request, sink, "load-manifest-parse-error", 0, 0);
        }
    };
    drop(parse_admission);

    let entry_count = load_manifest.plugins.len() as u64;
    let normalized_export_root = canonical_or_normalized(export_root.to_path_buf());
    for entry in load_manifest.plugins {
        request.check_active()?;
        let Some(manifest_path) = resolve_load_manifest_entry_path(
            request,
            sink,
            export_root,
            &normalized_export_root,
            &entry,
            "manifest",
            &entry.manifest,
        )?
        else {
            continue;
        };
        let candidate = match metered_candidate_from_manifest_path(request, sink, manifest_path) {
            Ok(candidate) => candidate,
            Err(
                error @ NativePluginDiscoveryRefreshError::BudgetExceeded { .. }
                | error @ NativePluginDiscoveryRefreshError::Cancelled
                | error @ NativePluginDiscoveryRefreshError::DeadlineExceeded,
            ) => return Err(error),
            Err(error) => {
                emit_diagnostic(request, sink, || error.to_string())?;
                continue;
            }
        };
        if !validate_load_manifest_entry(
            request,
            sink,
            export_root,
            &normalized_export_root,
            &entry,
            candidate.candidate(),
        )? {
            continue;
        }
        if sink.contains_candidate_id(&candidate.candidate().package_manifest.id) {
            let plugin_id = candidate.candidate().package_manifest.id.clone();
            emit_diagnostic(request, sink, || {
                format!("native plugin {plugin_id} load manifest duplicate package id ignored")
            })?;
            continue;
        }
        candidate.insert(sink);
    }
    input_identity(request, sink, "load-manifest", 0, entry_count)
}

fn emit_recoverable_error(
    request: &NativePluginDiscoveryRefreshRequest,
    sink: &mut NativePluginDiscoveryRefreshSink,
    error: NativePluginDiscoveryRefreshError,
    entry_count: u64,
) -> Result<
    super::discovery_refresh::NativePluginDiscoveryInputIdentity,
    NativePluginDiscoveryRefreshError,
> {
    match error {
        error @ NativePluginDiscoveryRefreshError::BudgetExceeded { .. }
        | error @ NativePluginDiscoveryRefreshError::Cancelled
        | error @ NativePluginDiscoveryRefreshError::DeadlineExceeded => Err(error),
        error => {
            emit_diagnostic(request, sink, || error.to_string())?;
            input_identity(request, sink, "load-manifest-read-error", 0, entry_count)
        }
    }
}

fn emit_diagnostic(
    request: &NativePluginDiscoveryRefreshRequest,
    sink: &mut NativePluginDiscoveryRefreshSink,
    build: impl FnOnce() -> String,
) -> Result<(), NativePluginDiscoveryRefreshError> {
    let reservation = sink.reserve_diagnostic(request)?;
    reservation.insert(sink, build());
    Ok(())
}

fn reserve_load_manifest_parse_scratch(
    request: &NativePluginDiscoveryRefreshRequest,
    sink: &mut NativePluginDiscoveryRefreshSink,
    source_bytes: u64,
) -> Result<NativePluginDiscoveryRefreshScratchReservation, NativePluginDiscoveryRefreshError> {
    // `Vec` capacity can round up while TOML owns decoded strings. Admit two source-sized copies
    // plus twice the fixed entry storage before deserialization makes either allocation.
    let entry_bytes = (request.budget().max_candidates as u64)
        .saturating_mul(2)
        .saturating_mul(std::mem::size_of::<NativePluginLoadManifestEntry>() as u64);
    let parse_bytes = source_bytes.saturating_add(entry_bytes);
    sink.reserve_additional_scratch_bytes(request, parse_bytes)
}

fn parse_bounded_load_manifest(
    source: &str,
    max_plugins: usize,
) -> Result<NativePluginLoadManifest, toml::de::Error> {
    BoundedLoadManifest { max_plugins }.deserialize(toml::de::Deserializer::parse(source)?)
}

struct BoundedLoadManifest {
    max_plugins: usize,
}

impl<'de> DeserializeSeed<'de> for BoundedLoadManifest {
    type Value = NativePluginLoadManifest;

    fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        deserializer.deserialize_map(BoundedLoadManifestVisitor {
            max_plugins: self.max_plugins,
        })
    }
}

struct BoundedLoadManifestVisitor {
    max_plugins: usize,
}

impl<'de> Visitor<'de> for BoundedLoadManifestVisitor {
    type Value = NativePluginLoadManifest;

    fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("a bounded native plugin load manifest table")
    }

    fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
    where
        A: MapAccess<'de>,
    {
        let mut plugins = Vec::new();
        while let Some(field) = map.next_key::<String>()? {
            if field == "plugins" {
                plugins = map.next_value_seed(BoundedLoadManifestEntries {
                    max_plugins: self.max_plugins,
                })?;
            } else {
                map.next_value::<IgnoredAny>()?;
            }
        }
        Ok(NativePluginLoadManifest { plugins })
    }
}

struct BoundedLoadManifestEntries {
    max_plugins: usize,
}

impl<'de> DeserializeSeed<'de> for BoundedLoadManifestEntries {
    type Value = Vec<NativePluginLoadManifestEntry>;

    fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        deserializer.deserialize_seq(self)
    }
}

impl<'de> Visitor<'de> for BoundedLoadManifestEntries {
    type Value = Vec<NativePluginLoadManifestEntry>;

    fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("a bounded native plugin load manifest entry sequence")
    }

    fn visit_seq<A>(self, mut sequence: A) -> Result<Self::Value, A::Error>
    where
        A: SeqAccess<'de>,
    {
        let mut plugins = Vec::new();
        plugins
            .try_reserve_exact(self.max_plugins)
            .map_err(|_| A::Error::custom("native plugin load manifest entry storage exhausted"))?;
        while plugins.len() < self.max_plugins {
            let Some(entry) = sequence.next_element::<NativePluginLoadManifestEntry>()? else {
                return Ok(plugins);
            };
            plugins.push(entry);
        }
        if sequence.next_element::<IgnoredAny>()?.is_some() {
            return Err(A::Error::custom(
                "native plugin load manifest exceeds the admitted candidate budget",
            ));
        }
        Ok(plugins)
    }
}

fn validate_load_manifest_entry(
    request: &NativePluginDiscoveryRefreshRequest,
    sink: &mut NativePluginDiscoveryRefreshSink,
    export_root: &Path,
    normalized_export_root: &Path,
    entry: &NativePluginLoadManifestEntry,
    candidate: &NativePluginCandidate,
) -> Result<bool, NativePluginDiscoveryRefreshError> {
    let mut accepted = true;
    if entry.id != candidate.package_manifest.id {
        emit_diagnostic(request, sink, || {
            format!(
                "native plugin {} load manifest id mismatch: entry id {}",
                candidate.package_manifest.id, entry.id
            )
        })?;
        accepted = false;
    }

    let Some(package_path) = resolve_load_manifest_entry_path(
        request,
        sink,
        export_root,
        normalized_export_root,
        entry,
        "path",
        &entry.path,
    )?
    else {
        return Ok(false);
    };
    let package_path = canonical_or_normalized(package_path);
    let manifest_parent = candidate
        .manifest_path
        .parent()
        .map(Path::to_path_buf)
        .unwrap_or_else(|| candidate.manifest_path.clone());
    let manifest_parent = canonical_or_normalized(manifest_parent);
    if !manifest_parent.starts_with(&package_path) {
        emit_diagnostic(request, sink, || {
            format!(
                "native plugin {} load manifest path mismatch: manifest {} is outside package path {}",
                candidate.package_manifest.id,
                candidate.manifest_path.display(),
                package_path.display()
            )
        })?;
        accepted = false;
    }
    Ok(accepted)
}

fn resolve_load_manifest_entry_path(
    request: &NativePluginDiscoveryRefreshRequest,
    sink: &mut NativePluginDiscoveryRefreshSink,
    export_root: &Path,
    normalized_export_root: &Path,
    entry: &NativePluginLoadManifestEntry,
    field_name: &str,
    field_path: &str,
) -> Result<Option<PathBuf>, NativePluginDiscoveryRefreshError> {
    let path = resolve_manifest_path(export_root, field_path);
    let normalized_path = canonical_or_normalized(path.clone());
    if !normalized_path.starts_with(normalized_export_root) {
        emit_diagnostic(request, sink, || {
            format!(
                "native plugin {} load manifest {} escapes export root: {}",
                entry.id, field_name, field_path
            )
        })?;
        return Ok(None);
    }
    Ok(Some(path))
}

fn canonical_or_normalized(path: PathBuf) -> PathBuf {
    path.canonicalize().unwrap_or_else(|_| normalize_path(path))
}

fn normalize_path(path: PathBuf) -> PathBuf {
    let mut normalized = PathBuf::new();
    for component in path.components() {
        match component {
            Component::CurDir => {}
            Component::ParentDir => {
                normalized.pop();
            }
            Component::Prefix(prefix) => normalized.push(prefix.as_os_str()),
            Component::RootDir => normalized.push(component.as_os_str()),
            Component::Normal(part) => normalized.push(part),
        }
    }
    normalized
}

#[cfg(test)]
mod tests {
    use super::parse_bounded_load_manifest;

    #[test]
    fn bounded_load_manifest_rejects_an_entry_before_unbounded_vector_growth() {
        let source = r#"
[[plugins]]
id = "weather"
path = "plugins/weather"
manifest = "plugins/weather/plugin.toml"

[[plugins]]
id = "climate"
path = "plugins/climate"
manifest = "plugins/climate/plugin.toml"
"#;

        let error = parse_bounded_load_manifest(source, 1)
            .expect_err("second selection entry must exceed the admitted candidate capacity");

        assert!(
            error
                .to_string()
                .contains("exceeds the admitted candidate budget")
        );
    }
}
