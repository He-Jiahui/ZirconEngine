use super::super::super::PluginCatalogGeneration;

/// Work performed while evaluating one catalog update candidate.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct RuntimePluginCatalogUpdateMetrics {
    pub candidate_projection_builds: usize,
    pub candidate_diagnostic_builds: usize,
    pub published_generations: usize,
    pub candidate_registration_rows: usize,
    pub candidate_feature_registration_rows: usize,
    pub candidate_registration_rows_indexed: usize,
    pub candidate_feature_registration_rows_indexed: usize,
}

/// Reports whether a candidate became the next catalog generation without mutating rejected state.
#[derive(Clone, Debug, PartialEq, Eq)]
#[must_use = "rejected catalog updates retain the last-good generation"]
pub struct RuntimePluginCatalogUpdateOutcome {
    published_generation: Option<PluginCatalogGeneration>,
    diagnostics: Vec<String>,
    metrics: RuntimePluginCatalogUpdateMetrics,
}

impl RuntimePluginCatalogUpdateOutcome {
    pub(super) fn unchanged() -> Self {
        Self {
            published_generation: None,
            diagnostics: Vec::new(),
            metrics: RuntimePluginCatalogUpdateMetrics::default(),
        }
    }

    pub(super) fn rejected(
        diagnostics: Vec<String>,
        metrics: RuntimePluginCatalogUpdateMetrics,
    ) -> Self {
        Self {
            published_generation: None,
            diagnostics,
            metrics,
        }
    }

    pub(super) fn published(
        catalog_generation: PluginCatalogGeneration,
        metrics: RuntimePluginCatalogUpdateMetrics,
    ) -> Self {
        Self {
            published_generation: Some(catalog_generation),
            diagnostics: Vec::new(),
            metrics,
        }
    }

    pub(super) fn generation_exhausted(metrics: RuntimePluginCatalogUpdateMetrics) -> Self {
        Self::rejected(
            vec!["runtime plugin catalog generation space is exhausted".to_string()],
            metrics,
        )
    }

    pub fn is_published(&self) -> bool {
        self.published_generation.is_some()
    }

    pub fn is_rejected(&self) -> bool {
        self.published_generation.is_none() && !self.diagnostics.is_empty()
    }

    pub fn is_unchanged(&self) -> bool {
        self.published_generation.is_none() && self.diagnostics.is_empty()
    }

    pub fn published_generation(&self) -> Option<PluginCatalogGeneration> {
        self.published_generation
    }

    pub fn diagnostics(&self) -> &[String] {
        &self.diagnostics
    }

    pub fn metrics(&self) -> RuntimePluginCatalogUpdateMetrics {
        self.metrics
    }
}
