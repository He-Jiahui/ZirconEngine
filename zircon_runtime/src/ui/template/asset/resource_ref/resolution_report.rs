use zircon_runtime_interface::ui::template::{
    UiResourceDependency, UiResourceDiagnosticSeverity, UiResourceRef,
};

use super::{UiResolvedUiResource, UiResourceResolveDiagnostic, UiResourceResolver};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct UiResolvedResourceDependency {
    pub dependency: UiResourceDependency,
    pub resolved: UiResolvedUiResource,
    pub diagnostic_indices: Vec<usize>,
}

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
    pub fn resolve_dependencies(
        &mut self,
        dependencies: &[UiResourceDependency],
    ) -> UiResourceResolutionReport {
        let mut resources = Vec::with_capacity(dependencies.len());

        for dependency in dependencies {
            let diagnostics_start = self.diagnostics().len();
            let resolved = self.resolve(&dependency.reference);
            let diagnostics_end = self.diagnostics().len();
            let diagnostic_indices = diagnostic_indices_for_resolution(
                &dependency.reference,
                &resolved,
                self.diagnostics(),
                diagnostics_start,
                diagnostics_end,
            );

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

fn diagnostic_indices_for_resolution(
    reference: &UiResourceRef,
    resolved: &UiResolvedUiResource,
    diagnostics: &[UiResourceResolveDiagnostic],
    diagnostics_start: usize,
    diagnostics_end: usize,
) -> Vec<usize> {
    if diagnostics_end > diagnostics_start {
        return (diagnostics_start..diagnostics_end).collect();
    }

    match resolved {
        UiResolvedUiResource::Placeholder {
            diagnostic_index, ..
        } => {
            let mut indices = diagnostics
                .iter()
                .enumerate()
                .filter_map(|(index, diagnostic)| {
                    if diagnostic_matches_reference(diagnostic, reference) {
                        Some(index)
                    } else {
                        None
                    }
                })
                .collect::<Vec<_>>();
            if indices.is_empty() {
                indices.push(*diagnostic_index);
            }
            indices
        }
        UiResolvedUiResource::Handle { .. } => Vec::new(),
    }
}

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
