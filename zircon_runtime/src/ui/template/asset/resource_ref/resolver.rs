//! 将 UI 资源声明映射到运行时资源注册表；渲染消费还需另行取得实际资源内容。
//! 热重载按主 URI 或回退 URI 驱逐缓存，同时保留已有诊断下标的有效性。

use std::collections::{BTreeMap, HashMap, HashSet};

use crate::core::resource::{
    ResourceKind, ResourceLocator, ResourceLocatorError, ResourceManager, ResourceRecord,
    ResourceScheme, UntypedResourceHandle,
};
use thiserror::Error;
use zircon_runtime_interface::ui::template::{
    UiResourceDiagnosticSeverity, UiResourceFallbackMode, UiResourceKind, UiResourceRef,
};

/// 供资源报告区分 URI 无效、注册缺失与种类不匹配，便于作者定位回退失败阶段。
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

/// 一批热重载 URI 的缓存驱逐回执；诊断保留数反映历史记录，不表示剩余故障数。
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct UiResourceResolverCacheInvalidationReport {
    pub requested_uris: Vec<String>,
    pub references_removed: usize,
    pub diagnostics_retained: usize,
}

/// 宿主将 UI 的 asset/project 命名空间接到运行时 scheme 或指定 package。
/// 缺省不映射这两个命名空间，避免把作者资源路径猜成运行时已注册资源。
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct UiResourceResolverSchemeMap {
    pub asset_scheme: Option<ResourceScheme>,
    pub asset_package_id: Option<String>,
    pub project_scheme: Option<ResourceScheme>,
    pub project_package_id: Option<String>,
}

impl UiResourceResolverSchemeMap {
    pub fn asset_to(mut self, scheme: ResourceScheme) -> Self {
        self.asset_scheme = Some(scheme);
        self
    }

    pub fn asset_to_package(mut self, package_id: impl Into<String>) -> Self {
        self.asset_scheme = Some(ResourceScheme::Package);
        self.asset_package_id = Some(package_id.into());
        self
    }

    pub fn project_to(mut self, scheme: ResourceScheme) -> Self {
        self.project_scheme = Some(scheme);
        self
    }

    pub fn project_to_package(mut self, package_id: impl Into<String>) -> Self {
        self.project_scheme = Some(ResourceScheme::Package);
        self.project_package_id = Some(package_id.into());
        self
    }
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
    scheme_map: UiResourceResolverSchemeMap,
    cache: BTreeMap<UiResourceRef, UiResolvedUiResource>,
    diagnostics: Vec<UiResourceResolveDiagnostic>,
}

impl UiResourceResolver {
    /// 与宿主共享已有资源管理器；此层只查注册记录，不触发资源加载。
    pub fn new(resource_manager: ResourceManager) -> Self {
        Self {
            resource_manager,
            scheme_map: UiResourceResolverSchemeMap::default(),
            cache: BTreeMap::new(),
            diagnostics: Vec::new(),
        }
    }

    // TODO: [CR-UI-TEMPLATE-RSC-0001] 确认更换映射是否仅允许首次解析前调用；缓存键未含映射且此处不清缓存；下一步补充已缓存引用更换映射的契约测试。
    /// 配置宿主命名空间映射；在已有解析结果时使用应先清理缓存。
    pub fn with_scheme_map(mut self, scheme_map: UiResourceResolverSchemeMap) -> Self {
        self.scheme_map = scheme_map;
        self
    }

    pub fn scheme_map(&self) -> &UiResourceResolverSchemeMap {
        &self.scheme_map
    }

    /// 按包含类型与回退策略的完整引用缓存结果；注册表变化后须由宿主显式失效。
    /// 失败返回可显示的占位项和诊断，不保证句柄背后的资源已加载。
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

    /// 清理解析结果但保留诊断历史，避免已发出的占位下标失效。
    pub fn clear_cache(&mut self) {
        self.cache.clear();
    }

    /// 为热重载驱逐受主资源或回退资源影响的引用；接受作者 URI 或映射后的运行时 URI。
    /// 请求 URI 会去空白并按首次出现去重，诊断历史继续由当前解析器持有。
    pub fn invalidate_uris<I, S>(&mut self, uris: I) -> UiResourceResolverCacheInvalidationReport
    where
        I: IntoIterator<Item = S>,
        S: AsRef<str>,
    {
        let requested_uris = unique_trimmed_uris(uris);
        let scheme_map = self.scheme_map.clone();
        let references_removed = if requested_uris.is_empty() {
            0
        } else {
            let requested_uri_set = requested_uris
                .iter()
                .map(String::as_str)
                .collect::<HashSet<&str>>();
            let before = self.cache.len();
            self.cache.retain(|reference, _| {
                !resource_reference_contains_any_uri(reference, &requested_uri_set, &scheme_map)
            });
            before.saturating_sub(self.cache.len())
        };

        UiResourceResolverCacheInvalidationReport {
            requested_uris,
            references_removed,
            diagnostics_retained: self.diagnostics.len(),
        }
    }

    // 回退成功仍是占位结果：主资源缺失应持续对作者可见，而非被成功回退掩盖。
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
        let locator = match runtime_lookup_for_ui_uri(uri, &self.scheme_map) {
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

    // 诊断只追加；结果中保存的位置因此在后续解析与缓存驱逐后仍有效。
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

fn unique_trimmed_uris<I, S>(uris: I) -> Vec<String>
where
    I: IntoIterator<Item = S>,
    S: AsRef<str>,
{
    let mut uri_order = HashMap::<String, usize>::new();
    for uri in uris {
        let uri = uri.as_ref().trim();
        if uri.is_empty() || uri_order.contains_key(uri) {
            continue;
        }
        let index = uri_order.len();
        uri_order.insert(uri.to_string(), index);
    }

    let mut ordered = vec![None; uri_order.len()];
    for (uri, index) in uri_order {
        ordered[index] = Some(uri);
    }
    ordered.into_iter().flatten().collect()
}

fn resource_reference_contains_any_uri(
    reference: &UiResourceRef,
    requested_uri_set: &HashSet<&str>,
    scheme_map: &UiResourceResolverSchemeMap,
) -> bool {
    resource_uri_matches_any_invalidation(&reference.uri, requested_uri_set, scheme_map)
        || reference.fallback.uri.as_deref().is_some_and(|fallback| {
            resource_uri_matches_any_invalidation(fallback, requested_uri_set, scheme_map)
        })
}

fn resource_uri_matches_any_invalidation(
    reference_uri: &str,
    requested_uri_set: &HashSet<&str>,
    scheme_map: &UiResourceResolverSchemeMap,
) -> bool {
    requested_uri_set.contains(reference_uri)
        || mapped_runtime_locator_string(reference_uri, scheme_map)
            .is_some_and(|mapped| requested_uri_set.contains(mapped.as_str()))
}

fn mapped_runtime_locator_string(
    uri: &str,
    scheme_map: &UiResourceResolverSchemeMap,
) -> Option<String> {
    match runtime_lookup_for_ui_uri(uri, scheme_map).ok()? {
        RuntimeResourceLookup::Locator(locator) => Some(locator.to_string()),
        RuntimeResourceLookup::UiAssetScheme => None,
    }
}

enum RuntimeResourceLookup {
    Locator(ResourceLocator),
    UiAssetScheme,
}

type UiResourceLookupResult<T> = std::result::Result<T, UiResourceLookupError>;

#[derive(Debug, Error)]
enum UiResourceLookupError {
    #[error(transparent)]
    ResourceLocator(#[from] ResourceLocatorError),
}

// UI 专用命名空间只能通过显式映射进入运行时；其他 URI 交给资源定位器的统一校验。
fn runtime_lookup_for_ui_uri(
    uri: &str,
    scheme_map: &UiResourceResolverSchemeMap,
) -> UiResourceLookupResult<RuntimeResourceLookup> {
    let trimmed = uri.trim();
    if let Some(remainder) = trimmed.strip_prefix("asset://") {
        return mapped_ui_locator(
            remainder,
            scheme_map.asset_scheme,
            scheme_map.asset_package_id.as_deref(),
        );
    }
    if let Some(remainder) = trimmed.strip_prefix("project://") {
        return mapped_ui_locator(
            remainder,
            scheme_map.project_scheme,
            scheme_map.project_package_id.as_deref(),
        );
    }

    Ok(RuntimeResourceLookup::Locator(ResourceLocator::parse(
        trimmed,
    )?))
}

fn mapped_ui_locator(
    remainder: &str,
    scheme: Option<ResourceScheme>,
    project_package_id: Option<&str>,
) -> UiResourceLookupResult<RuntimeResourceLookup> {
    let Some(scheme) = scheme else {
        return Ok(RuntimeResourceLookup::UiAssetScheme);
    };
    let path = match scheme {
        ResourceScheme::Package => {
            if let Some(package_id) = project_package_id {
                format!("{package_id}/{remainder}")
            } else {
                remainder.to_string()
            }
        }
        _ => remainder.to_string(),
    };
    let (path, label) = split_ui_locator_label(&path)?;
    Ok(RuntimeResourceLookup::Locator(ResourceLocator::new(
        scheme, path, label,
    )?))
}

fn split_ui_locator_label(value: &str) -> UiResourceLookupResult<(String, Option<String>)> {
    match value.split_once('#') {
        Some((_path, label)) if label.is_empty() => Err(ResourceLocatorError::EmptyLabel.into()),
        Some((path, label)) => Ok((path.to_string(), Some(label.to_string()))),
        None => Ok((value.to_string(), None)),
    }
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

#[cfg(test)]
#[path = "resolver/tests/hash_invalidation_tests.rs"]
mod hash_invalidation_tests;
