//! Host-side materialization of verified native editor contribution batches.

use std::collections::BTreeMap;

use zircon_runtime::plugin::native::{NativePluginEditorCommandBinding, NativePluginLoadReport};
use zircon_runtime_interface::SerializedContributionBatch;

use crate::core::editor_extension::EditorExtensionRegistry;
use crate::core::editor_operation::EditorOperationPath;
use crate::core::plugin::run_editor_plugin_boundary;

#[derive(Default)]
pub(super) struct NativeEditorContributionMaterialization {
    registrations: BTreeMap<String, NativeEditorContributionRegistration>,
}

#[derive(Default)]
struct NativeEditorContributionRegistration {
    extensions: EditorExtensionRegistry,
    native_command_bindings: BTreeMap<EditorOperationPath, NativePluginEditorCommandBinding>,
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
    ) -> (
        EditorExtensionRegistry,
        BTreeMap<EditorOperationPath, NativePluginEditorCommandBinding>,
        Vec<String>,
    ) {
        self.registrations
            .remove(package_id)
            .map(|registration| {
                (
                    registration.extensions,
                    registration.native_command_bindings,
                    registration.diagnostics,
                )
            })
            .unwrap_or_default()
    }

    fn materialize_batch(
        &mut self,
        package_id: &str,
        batch: &SerializedContributionBatch,
        bind_command: impl Fn(&str) -> Result<NativePluginEditorCommandBinding, String>,
    ) {
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
        let mut candidate_bindings = registration.native_command_bindings.clone();
        match run_editor_plugin_boundary(
            package_id,
            "serialized contribution materialization",
            || {
                crate::core::plugin::materialize_serialized_native_contribution_batch(
                    batch,
                    &mut candidate_extensions,
                    &mut candidate_bindings,
                    bind_command,
                )
                .map_err(|error| error.to_string())
            },
        ) {
            Ok(()) => {
                registration.extensions = candidate_extensions;
                registration.native_command_bindings = candidate_bindings;
            }
            Err(error) => fault_registration(registration, error.to_string()),
        }
    }
}

pub(super) fn materialize_native_editor_contributions(
    native_report: &NativePluginLoadReport,
    include_package: impl Fn(&str) -> bool,
) -> NativeEditorContributionMaterialization {
    let mut materialization = NativeEditorContributionMaterialization::default();
    for plugin in native_report
        .loaded()
        .iter()
        .filter(|plugin| include_package(&plugin.plugin_id))
    {
        let Some(batch) = plugin
            .editor_entry_report
            .as_ref()
            .and_then(|report| report.editor_contribution_batch.as_ref())
        else {
            continue;
        };
        materialization.materialize_batch(plugin.plugin_id.as_str(), batch, |command_id| {
            plugin
                .bind_editor_command(command_id)
                .map_err(|error| error.to_string())
        });
    }
    materialization
}

fn materialize_native_editor_contribution_batches<'a>(
    batches: impl IntoIterator<Item = (&'a str, &'a SerializedContributionBatch)>,
) -> NativeEditorContributionMaterialization {
    let mut materialization = NativeEditorContributionMaterialization::default();
    for (package_id, batch) in batches {
        materialization.materialize_batch(package_id, batch, |_| {
            Err("native plugin binding is unavailable in this materialization context".to_owned())
        });
    }
    materialization
}

fn fault_registration(registration: &mut NativeEditorContributionRegistration, diagnostic: String) {
    registration.extensions = EditorExtensionRegistry::default();
    registration.native_command_bindings.clear();
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

        let (extensions, bindings, diagnostics) =
            materialization.take_registration("fixture.editor");

        assert!(diagnostics.is_empty());
        assert!(bindings.is_empty());
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
        let (extensions, bindings, diagnostics) =
            materialization.take_registration("fixture.editor");

        assert!(extensions.views().is_empty());
        assert!(bindings.is_empty());
        assert_eq!(diagnostics.len(), 1);
    }
}
