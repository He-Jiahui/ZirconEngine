mod candidate_rows;
mod outcome;

use std::sync::Arc;

use crate::plugin::{RuntimePluginFeatureRegistrationReport, RuntimePluginRegistrationReport};

use super::super::derived_projection::RuntimePluginCatalogProjection;
use super::super::diagnostics::collect_catalog_diagnostics;
use super::super::RuntimePluginCatalog;
use candidate_rows::CandidateRows;

pub use outcome::{RuntimePluginCatalogUpdateMetrics, RuntimePluginCatalogUpdateOutcome};

/// Owns one mutable candidate until validation either publishes it or retains the last-good catalog.
#[must_use = "catalog updates must be published or explicitly discarded"]
pub struct RuntimePluginCatalogUpdate<'a> {
    catalog: &'a mut RuntimePluginCatalog,
    registration_candidate: Option<CandidateRows<RuntimePluginRegistrationReport, String>>,
    feature_registration_candidate:
        Option<CandidateRows<RuntimePluginFeatureRegistrationReport, (String, String)>>,
    operation_diagnostics: Vec<String>,
    changed: bool,
}

impl RuntimePluginCatalog {
    /// Starts an isolated structural update from the current catalog generation.
    pub fn update(&mut self) -> RuntimePluginCatalogUpdate<'_> {
        RuntimePluginCatalogUpdate {
            catalog: self,
            registration_candidate: None,
            feature_registration_candidate: None,
            operation_diagnostics: Vec::new(),
            changed: false,
        }
    }
}

impl RuntimePluginCatalogUpdate<'_> {
    pub fn append_registration(&mut self, registration: RuntimePluginRegistrationReport) {
        let package_id = registration.package_manifest.id.clone();
        self.registrations_mut().append(package_id, registration);
        self.changed = true;
    }

    pub fn replace_registration(&mut self, registration: RuntimePluginRegistrationReport) -> bool {
        let package_id = registration.package_manifest.id.clone();
        let replaced = self.registrations_mut().replace(&package_id, registration);
        if replaced {
            self.changed = true;
        } else {
            self.operation_diagnostics.push(format!(
                "cannot replace missing runtime plugin registration {package_id}"
            ));
        }
        replaced
    }

    pub fn remove_registration(&mut self, package_id: &str) -> bool {
        let removed = self.registrations_mut().remove(&package_id.to_string());
        if removed {
            self.changed = true;
        } else {
            self.operation_diagnostics.push(format!(
                "cannot remove missing runtime plugin registration {package_id}"
            ));
        }
        removed
    }

    pub fn append_feature_registration(
        &mut self,
        registration: RuntimePluginFeatureRegistrationReport,
    ) {
        let identity = feature_registration_identity(&registration);
        self.feature_registrations_mut()
            .append(identity, registration);
        self.changed = true;
    }

    pub fn replace_feature_registration(
        &mut self,
        registration: RuntimePluginFeatureRegistrationReport,
    ) -> bool {
        let (feature_id, provider_package_id) = feature_registration_identity(&registration);
        let identity = (feature_id.clone(), provider_package_id.clone());
        let replaced = self
            .feature_registrations_mut()
            .replace(&identity, registration);
        if replaced {
            self.changed = true;
        } else {
            self.operation_diagnostics.push(format!(
                "cannot replace missing runtime feature registration {feature_id}@{provider_package_id}"
            ));
        }
        replaced
    }

    pub fn remove_feature_registration(
        &mut self,
        feature_id: &str,
        provider_package_id: &str,
    ) -> bool {
        let identity = (feature_id.to_string(), provider_package_id.to_string());
        let removed = self.feature_registrations_mut().remove(&identity);
        if removed {
            self.changed = true;
        } else {
            self.operation_diagnostics.push(format!(
                "cannot remove missing runtime feature registration {feature_id}@{provider_package_id}"
            ));
        }
        removed
    }

    /// Builds one candidate projection and diagnostic view before changing published state.
    pub fn publish(mut self) -> RuntimePluginCatalogUpdateOutcome {
        if !self.changed && self.operation_diagnostics.is_empty() {
            return RuntimePluginCatalogUpdateOutcome::unchanged();
        }

        let candidate_registration_rows_indexed = self
            .registration_candidate
            .as_ref()
            .map_or(0, CandidateRows::source_rows_indexed);
        let candidate_feature_registration_rows_indexed = self
            .feature_registration_candidate
            .as_ref()
            .map_or(0, CandidateRows::source_rows_indexed);
        let candidate_registrations = self
            .registration_candidate
            .take()
            .map(CandidateRows::into_rows);
        let candidate_feature_registrations = self
            .feature_registration_candidate
            .take()
            .map(CandidateRows::into_rows);
        let registrations = candidate_registrations
            .as_deref()
            .unwrap_or(&self.catalog.registrations);
        let feature_registrations = candidate_feature_registrations
            .as_deref()
            .unwrap_or(&self.catalog.feature_registrations);
        let catalog_generation = self.catalog.catalog_generation.saturating_add(1);
        let projection_builds = self.catalog.projection_builds.saturating_add(1);
        let projection = Arc::new(RuntimePluginCatalogProjection::build(
            registrations,
            feature_registrations,
            catalog_generation,
            projection_builds,
        ));
        let mut diagnostics = self.operation_diagnostics;
        diagnostics.extend(collect_catalog_diagnostics(
            self.catalog.module_order_error.as_deref(),
            registrations,
            feature_registrations,
            projection.as_ref(),
        ));
        let metrics = RuntimePluginCatalogUpdateMetrics {
            candidate_projection_builds: 1,
            candidate_diagnostic_builds: 1,
            published_generations: if diagnostics.is_empty() { 1 } else { 0 },
            candidate_registration_rows: registrations.len(),
            candidate_feature_registration_rows: feature_registrations.len(),
            candidate_registration_rows_indexed,
            candidate_feature_registration_rows_indexed,
        };
        if !diagnostics.is_empty() {
            return RuntimePluginCatalogUpdateOutcome::rejected(diagnostics, metrics);
        }

        if let Some(registrations) = candidate_registrations {
            self.catalog.registrations = registrations;
        }
        if let Some(feature_registrations) = candidate_feature_registrations {
            self.catalog.feature_registrations = feature_registrations;
        }
        self.catalog.catalog_generation = catalog_generation;
        self.catalog.projection_builds = projection_builds;
        self.catalog.projection = projection;
        self.catalog.diagnostics.clear();
        self.catalog
            .project_plans
            .get_mut()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .clear();
        RuntimePluginCatalogUpdateOutcome::published(catalog_generation, metrics)
    }

    fn registrations_mut(&mut self) -> &mut CandidateRows<RuntimePluginRegistrationReport, String> {
        if self.registration_candidate.is_none() {
            self.registration_candidate = Some(CandidateRows::from_source(
                &self.catalog.registrations,
                |registration| registration.package_manifest.id.clone(),
            ));
        }
        self.registration_candidate
            .as_mut()
            .expect("registration candidate initialized")
    }

    fn feature_registrations_mut(
        &mut self,
    ) -> &mut CandidateRows<RuntimePluginFeatureRegistrationReport, (String, String)> {
        if self.feature_registration_candidate.is_none() {
            self.feature_registration_candidate = Some(CandidateRows::from_source(
                &self.catalog.feature_registrations,
                feature_registration_identity,
            ));
        }
        self.feature_registration_candidate
            .as_mut()
            .expect("feature registration candidate initialized")
    }
}

fn feature_registration_identity(
    registration: &RuntimePluginFeatureRegistrationReport,
) -> (String, String) {
    (
        registration.manifest.id.clone(),
        registration.provider_package_id_or_owner().to_string(),
    )
}

#[cfg(test)]
mod tests;
