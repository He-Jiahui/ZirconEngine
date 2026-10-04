use zircon_runtime_interface::{
    EditorCommandExecutionContract, EditorCommandResourceBudget, EditorCommandResultCodecId,
};

use super::{
    ensure_native_live_action_has_no_active_contribution,
    project_command_registry_from_contributions,
};
use crate::core::commands::{
    EditorCommandDescriptor, EditorCommandExecutorRegistryError, EditorCommandRegistryError,
};
use crate::core::editor_operation::EditorOperationPath;
use crate::core::extension::{ContributionBatch, ContributionSource, ContributionStore};
use crate::core::tools::ToolOwnerGeneration;
use crate::ui::workbench::shell_state::OwnedContribution;

#[test]
fn native_live_action_requires_a_generation_aware_contribution_transaction() {
    let mut contributions = ContributionStore::default();
    let ticket = contributions
        .contribute(ContributionSource::Builtin, ContributionBatch::default())
        .unwrap();
    let handle = crate::core::editor_extension::EditorContributionHandle::new(
        "fixture.native-editor",
        ticket,
        ToolOwnerGeneration::BUILTIN,
    );
    let owners = [OwnedContribution::new(handle)];

    let error =
        ensure_native_live_action_has_no_active_contribution(&owners, "fixture.native-editor")
            .expect_err("live action must not bypass an active exact contribution generation");

    assert!(error.contains("fixture.native-editor"));
    assert!(error.contains("generation-aware contribution transaction"));
    assert!(
        ensure_native_live_action_has_no_active_contribution(&owners, "fixture.runtime-only")
            .is_ok()
    );
}

#[test]
fn native_endpoint_without_same_batch_binding_is_rejected_before_projection() {
    let command_id = EditorOperationPath::parse("fixture.editor.native").unwrap();
    let descriptor = EditorCommandDescriptor::native(command_id).with_execution_contract(
        EditorCommandExecutionContract::new(
            EditorCommandResultCodecId::parse("zircon.editor.result.v1").unwrap(),
            EditorCommandResourceBudget::new(1024, 1024, 1000).unwrap(),
        ),
    );
    let mut batch = ContributionBatch::default();
    batch.register_command(descriptor).unwrap();
    let mut contributions = ContributionStore::default();
    contributions
        .contribute(ContributionSource::Builtin, batch)
        .unwrap();

    let error = project_command_registry_from_contributions(&contributions, 0)
        .expect_err("native endpoint without an admitted binding must not publish");
    assert!(matches!(
        error,
        crate::core::editor_extension::EditorExtensionRegistryError::Command(
            EditorCommandRegistryError::Executor(
                EditorCommandExecutorRegistryError::MissingExecutor { .. }
            )
        )
    ));
}

#[test]
fn native_binding_owner_must_match_contribution_owner() {
    let command_id = EditorOperationPath::parse("fixture.editor.native").unwrap();
    let error = super::validate_native_binding_owner(&command_id, "fixture.editor", "other.editor");
    assert!(matches!(
        error,
        crate::core::editor_extension::EditorExtensionRegistryError::View(message)
            if message.contains("other.editor")
                && message.contains("fixture.editor")
    ));
}
