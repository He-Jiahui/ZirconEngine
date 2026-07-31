use crate::plugin::{RuntimePluginFeatureRegistrationReport, RuntimePluginRegistrationReport};
use std::sync::Arc;

use super::super::derived_projection::RuntimePluginCatalogProjection;
use super::super::diagnostics::collect_catalog_diagnostics;
use super::super::RuntimePluginCatalog;

impl RuntimePluginCatalog {
    pub fn from_registration_reports(
        registrations: impl IntoIterator<Item = RuntimePluginRegistrationReport>,
        feature_registrations: impl IntoIterator<Item = RuntimePluginFeatureRegistrationReport>,
    ) -> Self {
        let mut catalog = Self::default();
        for registration in registrations {
            catalog.registrations.push(registration);
        }
        for registration in feature_registrations {
            catalog.feature_registrations.push(registration);
        }
        catalog.publish_initial_generation();
        catalog
    }

    pub(super) fn publish_initial_generation(&mut self) {
        let catalog_generation = self.catalog_generation.saturating_add(1);
        let projection_builds = self.projection_builds.saturating_add(1);
        let projection = Arc::new(RuntimePluginCatalogProjection::build(
            &self.registrations,
            &self.feature_registrations,
            catalog_generation,
            projection_builds,
        ));
        let diagnostics = collect_catalog_diagnostics(
            self.module_order_error.as_deref(),
            &self.registrations,
            &self.feature_registrations,
            projection.as_ref(),
        );

        self.catalog_generation = catalog_generation;
        self.projection_builds = projection_builds;
        self.projection = projection;
        self.diagnostics = diagnostics;
        self.project_plans
            .get_mut()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .clear();
    }
}
