use std::sync::Arc;

use crate::core::editing::operation::{
    DeferredOperationInvocation, OperationCommand, OperationCommandFactory,
    OperationCommandFactoryError, OperationCommandFactoryRegistration, PendingEditRetention,
};
use crate::core::editor_operation::EditorOperationInvocation;
use crate::core::notifications::{
    DecisionNotification, DecisionNotificationCenter, DecisionOption, DecisionOptionId,
    DecisionTicket, NotificationId, NotificationSource,
};

pub(super) fn publish_foreign_receipt(center: &DecisionNotificationCenter, label: &str) {
    let (ticket, acknowledge) = publish_foreign_pending(center, label);
    center.resolve(&ticket, &acknowledge).unwrap();
}

pub(super) fn publish_foreign_pending(
    center: &DecisionNotificationCenter,
    label: &str,
) -> (DecisionTicket, DecisionOptionId) {
    let acknowledge = DecisionOptionId::parse("acknowledge").unwrap();
    let dismiss = DecisionOptionId::parse("dismiss").unwrap();
    let notification = DecisionNotification::new(
        NotificationId::parse(format!("editor.test.foreign-decision.{label}")).unwrap(),
        NotificationSource::builtin("editor.test").unwrap(),
        "editor.test.foreign.title",
        "editor.test.foreign.message",
        vec![
            DecisionOption::new(acknowledge.clone(), "editor.test.foreign.acknowledge").unwrap(),
            DecisionOption::new(dismiss, "editor.test.foreign.dismiss").unwrap(),
        ],
    )
    .unwrap();
    let ticket = center.publish(notification).unwrap();
    (ticket, acknowledge)
}

pub(super) fn first_pending_selection(
    center: &DecisionNotificationCenter,
) -> (DecisionTicket, DecisionOptionId) {
    let snapshot = center
        .pending_snapshot()
        .into_iter()
        .next()
        .expect("a pending Decision should be available");
    let option = snapshot
        .notification()
        .options()
        .first()
        .expect("a Decision must contain at least two options");
    (snapshot.ticket().clone(), option.id().clone())
}

pub(super) fn resolve_first_pending(center: &DecisionNotificationCenter) {
    let (ticket, option) = first_pending_selection(center);
    center
        .resolve(&ticket, &option)
        .expect("the first pending core Decision should resolve");
}

pub(super) fn deferred_apply_failure(name: &str) -> DeferredOperationInvocation {
    let invocation = EditorOperationInvocation::parse(format!("editor.test.{name}"))
        .expect("test operation should be valid");
    OperationCommandFactoryRegistration::new(
        invocation.operation_id.clone(),
        "pending decision reconcile failure fixture",
        crate::core::editing::operation::EditOperationTarget::EditWorkspace,
        Arc::new(AlwaysFailOperationFactory),
    )
    .with_pending_edit_retention(PendingEditRetention::Lossless)
    .defer(invocation)
    .expect("fixture registration should bind the deferred test operation")
}

pub(super) struct AlwaysFailOperationFactory;

impl OperationCommandFactory for AlwaysFailOperationFactory {
    fn create(
        &self,
        invocation: &EditorOperationInvocation,
    ) -> Result<OperationCommand, OperationCommandFactoryError> {
        Err(OperationCommandFactoryError::Factory {
            operation: invocation.operation_id.clone(),
            reason: "injected apply failure".to_string(),
        })
    }
}
