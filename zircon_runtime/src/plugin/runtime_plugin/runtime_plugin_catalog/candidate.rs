use std::fmt;
use std::sync::Arc;

use crate::core::CoreError;
use crate::plugin::{RuntimePluginFeatureRegistrationReport, RuntimePluginRegistrationReport};

use super::registration::CandidateRows;
use super::{
    RuntimePluginCatalog, RuntimePluginCatalogSnapshot, RuntimePluginCatalogUpdateMetrics,
};

/// Mutable, unpublished rows rooted at one immutable catalog generation.
#[must_use = "catalog candidates must be prepared or explicitly discarded"]
pub struct RuntimePluginCatalogCandidate {
    base_snapshot: Arc<RuntimePluginCatalogSnapshot>,
    base_generation: super::PluginCatalogGeneration,
    projection_builds: u64,
    registrations: CandidateRows<RuntimePluginRegistrationReport, String>,
    feature_registrations: CandidateRows<RuntimePluginFeatureRegistrationReport, (String, String)>,
    module_order_error: Option<Arc<CoreError>>,
    operation_diagnostics: Vec<String>,
    changed: bool,
}

/// A validated, sealed successor that has not yet been published by the catalog authority.
#[derive(Debug)]
#[must_use = "prepared catalog generations must be published or explicitly discarded"]
pub struct RuntimePluginCatalogPreparedGeneration {
    base_snapshot: Arc<RuntimePluginCatalogSnapshot>,
    snapshot: Arc<RuntimePluginCatalogSnapshot>,
    metrics: RuntimePluginCatalogUpdateMetrics,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RuntimePluginCatalogPreparationError {
    diagnostics: Vec<String>,
    metrics: RuntimePluginCatalogUpdateMetrics,
}

impl RuntimePluginCatalogCandidate {
    pub(super) fn from_snapshot(snapshot: Arc<RuntimePluginCatalogSnapshot>) -> Self {
        let catalog = snapshot.catalog();
        Self {
            base_snapshot: Arc::clone(&snapshot),
            base_generation: snapshot.generation(),
            projection_builds: catalog.projection_builds,
            registrations: CandidateRows::from_source(&catalog.registrations, |registration| {
                registration.package_manifest.id.clone()
            }),
            feature_registrations: CandidateRows::from_source(
                &catalog.feature_registrations,
                feature_registration_identity,
            ),
            module_order_error: catalog.module_order_error.clone(),
            operation_diagnostics: Vec::new(),
            changed: false,
        }
    }

    pub fn append_registration(&mut self, registration: RuntimePluginRegistrationReport) {
        let package_id = registration.package_manifest.id.clone();
        self.registrations.append(package_id, registration);
        self.changed = true;
    }

    pub fn replace_registration(&mut self, registration: RuntimePluginRegistrationReport) -> bool {
        let package_id = registration.package_manifest.id.clone();
        let replaced = self.registrations.replace(&package_id, registration);
        if !replaced {
            self.operation_diagnostics.push(format!(
                "cannot replace missing runtime plugin registration {package_id}"
            ));
        } else {
            self.changed = true;
        }
        replaced
    }

    pub fn remove_registration(&mut self, package_id: &str) -> bool {
        let removed = self.registrations.remove(&package_id.to_string());
        if !removed {
            self.operation_diagnostics.push(format!(
                "cannot remove missing runtime plugin registration {package_id}"
            ));
        } else {
            self.changed = true;
        }
        removed
    }

    pub fn append_feature_registration(
        &mut self,
        registration: RuntimePluginFeatureRegistrationReport,
    ) {
        let identity = feature_registration_identity(&registration);
        self.feature_registrations.append(identity, registration);
        self.changed = true;
    }

    pub fn replace_feature_registration(
        &mut self,
        registration: RuntimePluginFeatureRegistrationReport,
    ) -> bool {
        let (feature_id, provider_package_id) = feature_registration_identity(&registration);
        let identity = (feature_id.clone(), provider_package_id.clone());
        let replaced = self.feature_registrations.replace(&identity, registration);
        if !replaced {
            self.operation_diagnostics.push(format!(
                "cannot replace missing runtime feature registration {feature_id}@{provider_package_id}"
            ));
        } else {
            self.changed = true;
        }
        replaced
    }

    pub fn remove_feature_registration(
        &mut self,
        feature_id: &str,
        provider_package_id: &str,
    ) -> bool {
        let identity = (feature_id.to_string(), provider_package_id.to_string());
        let removed = self.feature_registrations.remove(&identity);
        if !removed {
            self.operation_diagnostics.push(format!(
                "cannot remove missing runtime feature registration {feature_id}@{provider_package_id}"
            ));
        } else {
            self.changed = true;
        }
        removed
    }

    pub fn prepare(
        self,
    ) -> Result<RuntimePluginCatalogPreparedGeneration, RuntimePluginCatalogPreparationError> {
        let registration_rows_indexed = self.registrations.source_rows_indexed();
        let feature_registration_rows_indexed = self.feature_registrations.source_rows_indexed();
        let registrations = self.registrations.into_rows();
        let feature_registrations = self.feature_registrations.into_rows();
        let mut metrics = RuntimePluginCatalogUpdateMetrics {
            candidate_registration_rows: registrations.len(),
            candidate_feature_registration_rows: feature_registrations.len(),
            candidate_registration_rows_indexed: registration_rows_indexed,
            candidate_feature_registration_rows_indexed: feature_registration_rows_indexed,
            published_generations: 0,
            ..RuntimePluginCatalogUpdateMetrics::default()
        };
        if !self.changed && self.operation_diagnostics.is_empty() {
            return Err(RuntimePluginCatalogPreparationError::new(
                vec!["runtime plugin catalog candidate does not contain changes".to_string()],
                metrics,
            ));
        }
        let Some(candidate_generation) = self.base_generation.checked_next() else {
            return Err(RuntimePluginCatalogPreparationError::new(
                vec!["runtime plugin catalog generation space is exhausted".to_string()],
                metrics,
            ));
        };

        let mut catalog = RuntimePluginCatalog::default();
        catalog.registrations = registrations;
        catalog.feature_registrations = feature_registrations;
        catalog.catalog_generation = candidate_generation;
        catalog.projection_builds = self.projection_builds;
        catalog.module_order_error = self.module_order_error;
        catalog.publish_initial_generation();
        metrics.candidate_projection_builds = 1;
        metrics.candidate_diagnostic_builds = 1;

        let mut diagnostics = self.operation_diagnostics;
        diagnostics.extend(catalog.diagnostics.iter().cloned());
        if !diagnostics.is_empty() {
            return Err(RuntimePluginCatalogPreparationError::new(
                diagnostics,
                metrics,
            ));
        }

        Ok(RuntimePluginCatalogPreparedGeneration {
            base_snapshot: self.base_snapshot,
            snapshot: Arc::new(RuntimePluginCatalogSnapshot::from_catalog(catalog)),
            metrics,
        })
    }
}

impl RuntimePluginCatalogPreparedGeneration {
    pub fn snapshot(&self) -> &Arc<RuntimePluginCatalogSnapshot> {
        &self.snapshot
    }

    pub fn metrics(&self) -> RuntimePluginCatalogUpdateMetrics {
        self.metrics
    }

    pub fn into_snapshot(self) -> Arc<RuntimePluginCatalogSnapshot> {
        self.snapshot
    }

    pub(super) fn into_publication_parts(
        self,
    ) -> (
        Arc<RuntimePluginCatalogSnapshot>,
        Arc<RuntimePluginCatalogSnapshot>,
    ) {
        (self.base_snapshot, self.snapshot)
    }
}

impl RuntimePluginCatalogPreparationError {
    fn new(diagnostics: Vec<String>, metrics: RuntimePluginCatalogUpdateMetrics) -> Self {
        Self {
            diagnostics,
            metrics,
        }
    }

    pub fn diagnostics(&self) -> &[String] {
        &self.diagnostics
    }

    pub fn metrics(&self) -> RuntimePluginCatalogUpdateMetrics {
        self.metrics
    }
}

impl fmt::Display for RuntimePluginCatalogPreparationError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{}", self.diagnostics.join("; "))
    }
}

impl std::error::Error for RuntimePluginCatalogPreparationError {}

fn feature_registration_identity(
    registration: &RuntimePluginFeatureRegistrationReport,
) -> (String, String) {
    (
        registration.manifest.id.clone(),
        registration.provider_package_id_or_owner().to_string(),
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::plugin::PluginPackageManifest;

    #[test]
    fn prepared_candidate_advances_once_without_mutating_the_base_snapshot() {
        let base = Arc::new(RuntimePluginCatalogSnapshot::from_catalog(
            RuntimePluginCatalog::from_registration_reports([package_report("base")], []),
        ));
        let mut candidate = base.stage_update();
        candidate.append_registration(package_report("next"));

        let prepared = candidate.prepare().expect("candidate should prepare");

        assert_eq!(base.generation().get(), 1);
        assert_eq!(prepared.snapshot().generation().get(), 2);
        assert_eq!(base.catalog().registrations().len(), 1);
        assert_eq!(prepared.snapshot().catalog().registrations().len(), 2);
        assert_eq!(prepared.metrics().candidate_registration_rows_indexed, 1);
        assert_eq!(
            prepared
                .metrics()
                .candidate_feature_registration_rows_indexed,
            0
        );
        assert_eq!(prepared.metrics().candidate_projection_builds, 1);
        assert_eq!(prepared.metrics().candidate_diagnostic_builds, 1);
        assert_eq!(prepared.metrics().published_generations, 0);
    }

    #[test]
    fn empty_candidate_cannot_consume_a_generation() {
        let base = Arc::new(RuntimePluginCatalogSnapshot::from_catalog(
            RuntimePluginCatalog::from_registration_reports([package_report("base")], []),
        ));

        let error = base
            .stage_update()
            .prepare()
            .expect_err("empty candidate should not prepare");

        assert!(error
            .diagnostics()
            .iter()
            .any(|diagnostic| diagnostic.contains("does not contain changes")));
        assert_eq!(base.generation().get(), 1);
    }

    #[test]
    fn invalid_candidate_keeps_the_base_snapshot_available() {
        let base = Arc::new(RuntimePluginCatalogSnapshot::from_catalog(
            RuntimePluginCatalog::from_registration_reports([package_report("base")], []),
        ));
        let mut candidate = base.stage_update();
        candidate.append_registration(package_report("base"));

        let error = candidate
            .prepare()
            .expect_err("duplicate package should not prepare");

        assert!(!error.diagnostics().is_empty());
        assert_eq!(base.catalog().registrations().len(), 1);
        assert_eq!(base.generation().get(), 1);
    }

    fn package_report(package_id: &str) -> RuntimePluginRegistrationReport {
        RuntimePluginRegistrationReport::from_native_package_manifest(PluginPackageManifest::new(
            package_id, package_id,
        ))
    }
}
