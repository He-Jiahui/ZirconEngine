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

    let (extensions, bindings, diagnostics) = materialization.take_registration("fixture.editor");

    assert!(diagnostics.is_empty());
    assert!(bindings.is_empty());
    assert_eq!(extensions.views().len(), 1);
    assert_eq!(extensions.views()[0].id(), "fixture.editor.view");
}

#[test]
fn only_successfully_materialized_batches_are_usable_for_registration() {
    let batch = view_batch("fixture.editor", "fixture.editor.view");
    let empty = NativeEditorContributionMaterialization::default();
    let materialized = materialize_native_editor_contribution_batches([("fixture.editor", &batch)]);

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
    let (extensions, bindings, diagnostics) = materialization.take_registration("fixture.editor");

    assert!(extensions.views().is_empty());
    assert!(bindings.is_empty());
    assert_eq!(diagnostics.len(), 1);
}
