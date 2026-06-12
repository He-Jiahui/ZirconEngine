use std::collections::BTreeMap;

use crate::core::resource::{
    ResourceKind, ResourceLocator, ResourceManager, ResourceRecord, UntypedResourceHandle,
};
use zircon_runtime_interface::ui::template::{
    UiResourceDiagnosticSeverity, UiResourceFallbackMode, UiResourceKind, UiResourceRef,
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum UiResourceResolveDiagnosticCode {
    InvalidUri,
    MissingPrimary,
    MissingFallback,
    KindMismatch,
}

/// Runtime-facing diagnostic for a template resource reference lookup.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct UiResourceResolveDiagnostic {
    pub code: UiResourceResolveDiagnosticCode,
    pub severity: UiResourceDiagnosticSeverity,
    pub uri: String,
    pub message: String,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct UiResourceResolverCacheInvalidationReport {
    pub requested_uris: Vec<String>,
    pub references_removed: usize,
    pub diagnostics_retained: usize,
}

/// Consumer-level resolution result for a UI template resource reference.
///
/// This layer resolves to the runtime resource registry's untyped handle. Later
/// renderer-specific layers map the handle to atlas slots, texture views, or
/// shaped font resources.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum UiResolvedUiResource {
    Handle {
        handle: UntypedResourceHandle,
        uri: String,
    },
    Placeholder {
        handle: Option<UntypedResourceHandle>,
        diagnostic_index: usize,
    },
}

/// Resolves template resource refs against the runtime resource manager.
///
/// The resolver is deliberately non-panicking: missing or incompatible resources
/// become placeholders with diagnostics so the editor can render a visible
/// fallback and surface the issue to authoring tools.
#[derive(Clone, Debug)]
pub struct UiResourceResolver {
    resource_manager: ResourceManager,
    cache: BTreeMap<UiResourceRef, UiResolvedUiResource>,
    diagnostics: Vec<UiResourceResolveDiagnostic>,
}

impl UiResourceResolver {
    pub fn new(resource_manager: ResourceManager) -> Self {
        Self {
            resource_manager,
            cache: BTreeMap::new(),
            diagnostics: Vec::new(),
        }
    }

    pub fn resolve(&mut self, reference: &UiResourceRef) -> UiResolvedUiResource {
        if let Some(resolved) = self.cache.get(reference) {
            return resolved.clone();
        }

        let resolved = self.resolve_uncached(reference);
        self.cache.insert(reference.clone(), resolved.clone());
        resolved
    }

    pub fn diagnostics(&self) -> &[UiResourceResolveDiagnostic] {
        &self.diagnostics
    }

    pub fn cache_len(&self) -> usize {
        self.cache.len()
    }

    pub fn clear_cache(&mut self) {
        self.cache.clear();
    }

    pub fn invalidate_uris<I, S>(&mut self, uris: I) -> UiResourceResolverCacheInvalidationReport
    where
        I: IntoIterator<Item = S>,
        S: AsRef<str>,
    {
        let mut requested_uris = Vec::new();
        let mut references_removed = 0;
        for uri in uris {
            let uri = uri.as_ref().trim();
            if uri.is_empty() || requested_uris.iter().any(|existing| existing == uri) {
                continue;
            }
            requested_uris.push(uri.to_string());
            let before = self.cache.len();
            self.cache
                .retain(|reference, _| !resource_reference_contains_uri(reference, uri));
            references_removed += before.saturating_sub(self.cache.len());
        }

        UiResourceResolverCacheInvalidationReport {
            requested_uris,
            references_removed,
            diagnostics_retained: self.diagnostics.len(),
        }
    }

    fn resolve_uncached(&mut self, reference: &UiResourceRef) -> UiResolvedUiResource {
        match self.resolve_uri(&reference.uri, reference.kind) {
            Ok(handle) => UiResolvedUiResource::Handle {
                handle,
                uri: reference.uri.clone(),
            },
            Err(primary_index) => match reference.fallback.mode {
                UiResourceFallbackMode::Placeholder => {
                    let fallback = reference
                        .fallback
                        .uri
                        .as_deref()
                        .and_then(|uri| self.resolve_fallback_uri(uri, reference.kind).ok());
                    UiResolvedUiResource::Placeholder {
                        handle: fallback,
                        diagnostic_index: primary_index,
                    }
                }
                UiResourceFallbackMode::None | UiResourceFallbackMode::Optional => {
                    UiResolvedUiResource::Placeholder {
                        handle: None,
                        diagnostic_index: primary_index,
                    }
                }
            },
        }
    }

    fn resolve_uri(
        &mut self,
        uri: &str,
        expected_kind: UiResourceKind,
    ) -> Result<UntypedResourceHandle, usize> {
        self.resolve_uri_with_missing_code(
            uri,
            expected_kind,
            UiResourceResolveDiagnosticCode::MissingPrimary,
            UiResourceDiagnosticSeverity::Warning,
            "resource uri",
        )
    }

    fn resolve_uri_with_missing_code(
        &mut self,
        uri: &str,
        expected_kind: UiResourceKind,
        missing_code: UiResourceResolveDiagnosticCode,
        missing_severity: UiResourceDiagnosticSeverity,
        context: &str,
    ) -> Result<UntypedResourceHandle, usize> {
        let locator = match runtime_lookup_for_ui_uri(uri) {
            Ok(RuntimeResourceLookup::Locator(locator)) => locator,
            Ok(RuntimeResourceLookup::UiAssetScheme) => {
                return Err(self.push_diagnostic(
                    missing_code,
                    missing_severity,
                    uri,
                    format!(
                        "{context} {uri} uses a UI asset scheme that is not registered in the runtime resource manager"
                    ),
                ))
            }
            Err(error) => {
                return Err(self.push_diagnostic(
                    UiResourceResolveDiagnosticCode::InvalidUri,
                    UiResourceDiagnosticSeverity::Error,
                    uri,
                    format!("{context} is invalid: {error}"),
                ))
            }
        };
        let Some(record) = self.record_for_locator(&locator) else {
            return Err(self.push_diagnostic(
                missing_code,
                missing_severity,
                uri,
                format!("{context} {uri} is not registered"),
            ));
        };
        let expected_resource_kind = resource_kind_for_ui_resource(expected_kind);
        if record.kind != expected_resource_kind {
            return Err(self.push_diagnostic(
                UiResourceResolveDiagnosticCode::KindMismatch,
                UiResourceDiagnosticSeverity::Error,
                uri,
                format!(
                    "{context} {uri} is registered as {:?}, expected {:?}",
                    record.kind, expected_resource_kind
                ),
            ));
        }
        Ok(UntypedResourceHandle::new(record.id, record.kind))
    }

    fn record_for_locator(&self, locator: &ResourceLocator) -> Option<ResourceRecord> {
        self.resource_manager
            .registry()
            .get_by_locator(locator)
            .cloned()
    }

    fn resolve_fallback_uri(
        &mut self,
        uri: &str,
        expected_kind: UiResourceKind,
    ) -> Result<UntypedResourceHandle, usize> {
        self.resolve_uri_with_missing_code(
            uri,
            expected_kind,
            UiResourceResolveDiagnosticCode::MissingFallback,
            UiResourceDiagnosticSeverity::Error,
            "placeholder fallback resource uri",
        )
    }

    fn push_diagnostic(
        &mut self,
        code: UiResourceResolveDiagnosticCode,
        severity: UiResourceDiagnosticSeverity,
        uri: &str,
        message: String,
    ) -> usize {
        let index = self.diagnostics.len();
        self.diagnostics.push(UiResourceResolveDiagnostic {
            code,
            severity,
            uri: uri.to_string(),
            message,
        });
        index
    }
}

fn resource_reference_contains_uri(reference: &UiResourceRef, uri: &str) -> bool {
    reference.uri == uri || reference.fallback.uri.as_deref() == Some(uri)
}

enum RuntimeResourceLookup {
    Locator(ResourceLocator),
    UiAssetScheme,
}

fn runtime_lookup_for_ui_uri(uri: &str) -> Result<RuntimeResourceLookup, String> {
    let trimmed = uri.trim();
    if has_ui_asset_scheme(trimmed) {
        return Ok(RuntimeResourceLookup::UiAssetScheme);
    }

    ResourceLocator::parse(trimmed)
        .map(RuntimeResourceLookup::Locator)
        .map_err(|error| error.to_string())
}

fn has_ui_asset_scheme(uri: &str) -> bool {
    uri.starts_with("asset://") || uri.starts_with("project://")
}

fn resource_kind_for_ui_resource(kind: UiResourceKind) -> ResourceKind {
    match kind {
        UiResourceKind::Font => ResourceKind::Font,
        UiResourceKind::Image => ResourceKind::Texture,
        UiResourceKind::Media | UiResourceKind::GenericAsset => ResourceKind::Data,
    }
}

impl Default for UiResourceResolver {
    fn default() -> Self {
        Self::new(ResourceManager::new())
    }
}
