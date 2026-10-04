use super::TaskCancellationToken;

#[test]
fn cloned_tokens_share_one_task_bound_cancellation_state() {
    let owner = TaskCancellationToken::new(17);
    let worker = owner.clone();

    assert_eq!(worker.task_id(), 17);
    assert!(!worker.is_cancellation_requested());

    owner.request_cancellation();

    assert!(worker.is_cancellation_requested());
}
