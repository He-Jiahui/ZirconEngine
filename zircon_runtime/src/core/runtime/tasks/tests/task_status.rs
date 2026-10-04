use super::*;

#[test]
fn runtime_task_status_has_one_terminal_state_and_no_poll_clock() {
    let mut status = TaskStatus::pending(TaskId::new(42));
    assert_eq!(status.state, TaskState::Pending);
    assert!(!status.is_terminal());

    status.mark_running();
    assert_eq!(status.state, TaskState::Running);

    status.mark_failed("worker panicked");
    assert_eq!(status.state, TaskState::Failed);
    assert_eq!(status.failure_message.as_deref(), Some("worker panicked"));
    assert!(status.is_terminal());
}
