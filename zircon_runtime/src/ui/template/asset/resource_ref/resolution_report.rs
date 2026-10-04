//! 将资源引用、占位结果和诊断下标组合成调用者可保存的批量报告。
//! 诊断快照包含解析器的完整历史，使缓存占位结果中的下标仍可直接查找。

use std::collections::HashMap;

use zircon_runtime_interface::ui::template::{
    UiResourceDependency, UiResourceDiagnosticSeverity, UiResourceRef,
};

use super::{UiResolvedUiResource, UiResourceResolveDiagnostic, UiResourceResolver};

/// 一个作者来源与它的解析结果；诊断下标指向同一报告的诊断快照，不能跨报告复用。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct UiResolvedResourceDependency {
    pub dependency: UiResourceDependency,
    pub resolved: UiResolvedUiResource,
    pub diagnostic_indices: Vec<usize>,
}

/// 批量解析的可独立持有快照；错误判断也包含解析器此前留下的错误。
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct UiResourceResolutionReport {
    pub resources: Vec<UiResolvedResourceDependency>,
    pub diagnostics: Vec<UiResourceResolveDiagnostic>,
}

impl UiResourceResolutionReport {
    pub fn resolved_count(&self) -> usize {
        self.resources
            .iter()
            .filter(|resource| matches!(resource.resolved, UiResolvedUiResource::Handle { .. }))
            .count()
    }

    pub fn placeholder_count(&self) -> usize {
        self.resources
            .iter()
            .filter(|resource| {
                matches!(resource.resolved, UiResolvedUiResource::Placeholder { .. })
            })
            .count()
    }

    pub fn has_errors(&self) -> bool {
        self.diagnostics
            .iter()
            .any(|diagnostic| diagnostic.severity == UiResourceDiagnosticSeverity::Error)
    }
}

impl UiResourceResolver {
    /// 按调用者顺序保留依赖，即使多个位置复用同一引用也分别报告。
    /// 缓存命中的占位项按 URI 关联历史诊断；使用新解析器可隔离不同诊断会话。
    pub fn resolve_dependencies(
        &mut self,
        dependencies: &[UiResourceDependency],
    ) -> UiResourceResolutionReport {
        let mut resources = Vec::with_capacity(dependencies.len());
        let mut diagnostic_indices_by_uri = diagnostic_index_by_uri(self.diagnostics());

        for dependency in dependencies {
            let diagnostics_start = self.diagnostics().len();
            let resolved = self.resolve(&dependency.reference);
            let diagnostics_end = self.diagnostics().len();
            let diagnostic_indices = if diagnostics_end > diagnostics_start {
                for index in diagnostics_start..diagnostics_end {
                    index_diagnostic(
                        &mut diagnostic_indices_by_uri,
                        &self.diagnostics()[index].uri,
                        index,
                    );
                }
                (diagnostics_start..diagnostics_end).collect()
            } else {
                match &resolved {
                    UiResolvedUiResource::Placeholder {
                        diagnostic_index, ..
                    } => diagnostic_indices_for_cached_resolution(
                        &dependency.reference,
                        *diagnostic_index,
                        &diagnostic_indices_by_uri,
                    ),
                    UiResolvedUiResource::Handle { .. } => Vec::new(),
                }
            };

            resources.push(UiResolvedResourceDependency {
                dependency: dependency.clone(),
                resolved,
                diagnostic_indices,
            });
        }

        UiResourceResolutionReport {
            resources,
            diagnostics: self.diagnostics().to_vec(),
        }
    }
}

fn diagnostic_index_by_uri(
    diagnostics: &[UiResourceResolveDiagnostic],
) -> HashMap<String, Vec<usize>> {
    let mut diagnostic_indices_by_uri = HashMap::new();
    for (index, diagnostic) in diagnostics.iter().enumerate() {
        index_diagnostic(&mut diagnostic_indices_by_uri, &diagnostic.uri, index);
    }
    diagnostic_indices_by_uri
}

fn index_diagnostic(
    diagnostic_indices_by_uri: &mut HashMap<String, Vec<usize>>,
    uri: &str,
    index: usize,
) {
    if let Some(indices) = diagnostic_indices_by_uri.get_mut(uri) {
        indices.push(index);
    } else {
        diagnostic_indices_by_uri.insert(uri.to_string(), vec![index]);
    }
}

// 合并主 URI 与回退 URI 的历史位置后恢复全局顺序；二者相同不得重复返回下标。
fn diagnostic_indices_for_cached_resolution(
    reference: &UiResourceRef,
    diagnostic_index: usize,
    diagnostic_indices_by_uri: &HashMap<String, Vec<usize>>,
) -> Vec<usize> {
    let mut indices = diagnostic_indices_by_uri
        .get(reference.uri.as_str())
        .into_iter()
        .flatten()
        .copied()
        .collect::<Vec<_>>();
    if let Some(fallback_uri) = reference.fallback.uri.as_deref() {
        if fallback_uri != reference.uri.as_str() {
            if let Some(fallback_indices) = diagnostic_indices_by_uri.get(fallback_uri) {
                indices.extend(fallback_indices.iter().copied());
            }
        }
    }
    if indices.is_empty() {
        indices.push(diagnostic_index);
    } else if reference.fallback.uri.is_some() {
        indices.sort_unstable();
        indices.dedup();
    }
    indices
}

#[cfg(test)]
fn diagnostic_matches_reference(
    diagnostic: &UiResourceResolveDiagnostic,
    reference: &UiResourceRef,
) -> bool {
    diagnostic.uri == reference.uri
        || reference
            .fallback
            .uri
            .as_deref()
            .is_some_and(|fallback_uri| diagnostic.uri == fallback_uri)
}

#[cfg(test)]
#[path = "resolution_report/tests/cached_diagnostic_index_tests.rs"]
mod cached_diagnostic_index_tests;
