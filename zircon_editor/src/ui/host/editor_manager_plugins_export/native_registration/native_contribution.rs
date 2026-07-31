//! Host-side materialization of verified native editor contribution batches.

use std::collections::BTreeMap;

use zircon_runtime::plugin::native::NativePluginLoadReport;
use zircon_runtime_interface::SerializedContributionBatch;

use crate::core::editor_extension::EditorExtensionRegistry;
use crate::core::plugin::{materialize_serialized_contribution_batch, run_editor_plugin_boundary};

#[derive(Default)]
pub(super) struct NativeEditorContributionMaterialization {
    registrations: BTreeMap<String, NativeEditorContributionRegistration>,
}

#[derive(Default)]
struct NativeEditorContributionRegistration {
    extensions: EditorExtensionRegistry,
    diagnostics: Vec<String>,
    faulted: bool,
}

impl NativeEditorContributionMaterialization {
    pub(super) fn is_registration_usable(&self, package_id: &str) -> bool {
        self.registrations
            .get(package_id)
            .is_some_and(|registration| !registration.faulted)
    }

    #[cfg(test)]
    pub(super) fn is_registration_faulted(&self, package_id: &str) -> bool {
        self.registrations
            .get(package_id)
            .is_some_and(|registration| registration.faulted)
    }

    pub(super) fn take_registration(
        &mut self,
        package_id: &str,
    ) -> (EditorExtensionRegistry, Vec<String>) {
        self.registrations
            .remove(package_id)
            .map(|registration| (registration.extensions, registration.diagnostics))
            .unwrap_or_default()
    }

    fn materialize_batch(&mut self, package_id: &str, batch: &SerializedContributionBatch) {
        let registration = self
            .registrations
            .entry(package_id.to_string())
            .or_default();
        if registration.faulted {
            return;
        }
        if batch.package_id() != package_id {
            fault_registration(
                registration,
                format!(
                    "serialized editor contribution package `{}` does not match native plugin `{package_id}`",
                    batch.package_id()
                ),
            );
            return;
        }

        let mut candidate_extensions = registration.extensions.clone();
        match run_editor_plugin_boundary(
            package_id,
            "serialized contribution materialization",
            || {
                materialize_serialized_contribution_batch(batch, &mut candidate_extensions)
                    .map_err(|error| error.to_string())
            },
        ) {
            Ok(()) => registration.extensions = candidate_extensions,
            Err(error) => fault_registration(registration, error.to_string()),
        }
    }
}

pub(super) fn materialize_native_editor_contributions(
    native_report: &NativePluginLoadReport,
    include_package: impl Fn(&str) -> bool,
) -> NativeEditorContributionMaterialization {
    materialize_native_editor_contribution_batches(native_report.loaded().iter().filter_map(
        |plugin| {
            if !include_package(&plugin.plugin_id) {
                return None;
            }
            plugin
                .editor_entry_report
                .as_ref()?
                .editor_contribution_batch
                .as_ref()
                .map(|batch| (plugin.plugin_id.as_str(), batch))
        },
    ))
}

fn materialize_native_editor_contribution_batches<'a>(
    batches: impl IntoIterator<Item = (&'a str, &'a SerializedContributionBatch)>,
) -> NativeEditorContributionMaterialization {
    let mut materialization = NativeEditorContributionMaterialization::default();
    for (package_id, batch) in batches {
        materialization.materialize_batch(package_id, batch);
    }
    materialization
}

fn fault_registration(registration: &mut NativeEditorContributionRegistration, diagnostic: String) {
    registration.extensions = EditorExtensionRegistry::default();
    registration.diagnostics.push(diagnostic);
    registration.faulted = true;
}

#[cfg(test)]
mod tests {
    use zircon_runtime_interface::{SerializedContributionBatch, SerializedEditorContribution};

    use super::{
        materialize_native_editor_contribution_batches, NativeEditorContributionMaterialization,
    };

    fn view_batch(package_id: &str, view_id: &str) -> SerializedContributionBatch {
        SerializedContributionBatch::new(
            package_id,
            vec![SerializedEditorContribution::View {
                id: view_id.to_string(),
                schema: SerializedEditorContribution::VIEW_SCHEMA.to_string(),
                title: "Fixture view".to_string(),
                category: "Tests".to_string(),
            }],
        )
        .expect("fixture contribution batch should be valid")
    }

    #[test]
    fn verified_native_batch_materializes_into_the_matching_package_registry() {
        let batch = view_batch("fixture.editor", "fixture.editor.view");
        let mut materialization =
            materialize_native_editor_contribution_batches([("fixture.editor", &batch)]);

        let (extensions, diagnostics) = materialization.take_registration("fixture.editor");

        assert!(diagnostics.is_empty());
        assert_eq!(extensions.views().len(), 1);
        assert_eq!(extensions.views()[0].id(), "fixture.editor.view");
    }

    #[test]
    fn only_successfully_materialized_batches_are_usable_for_registration() {
        let batch = view_batch("fixture.editor", "fixture.editor.view");
        let empty = NativeEditorContributionMaterialization::default();
        let materialized =
            materialize_native_editor_contribution_batches([("fixture.editor", &batch)]);

        assert!(!empty.is_registration_usable("fixture.editor"));
        assert!(materialized.is_registration_usable("fixture.editor"));
    }

    #[test]
    fn materialization_failure_revokes_all_prior_package_contributions() {
        let first = view_batch("fixture.editor", "fixture.editor.view");
        let duplicate = view_batch("fixture.editor", "fixture.editor.view");
        let mut materialization = materialize_native_editor_contribution_batches([
            ("fixture.editor", &first),
            ("fixture.editor", &duplicate),
        ]);

        assert!(materialization.is_registration_faulted("fixture.editor"));
        assert!(!materialization.is_registration_usable("fixture.editor"));
        let (extensions, diagnostics) = materialization.take_registration("fixture.editor");

        assert!(extensions.views().is_empty());
        assert_eq!(diagnostics.len(), 1);
    }
}
