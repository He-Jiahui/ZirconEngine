use super::*;

#[test]
fn action_history_keeps_newest_records() {
    let mut history = Vec::new();

    for index in 0..20 {
        push_action_record(
            &mut history,
            HubActionRecord {
                finished_unix_ms: index,
                action: HubActionKind::OpenEditor,
                status: HubActionStatus::Success,
                target: format!("target {index}"),
                detail: HubMessage::raw_text("opened"),
                log_excerpt: HubMessage::empty(),
                recovery: None,
                process_id: Some(index as u32),
                command_line: Vec::new(),
                output_dir: None,
            },
        );
    }

    assert_eq!(history.len(), ACTION_HISTORY_LIMIT);
    assert_eq!(history[0].target, "target 19");
    assert_eq!(history[ACTION_HISTORY_LIMIT - 1].target, "target 4");
}
