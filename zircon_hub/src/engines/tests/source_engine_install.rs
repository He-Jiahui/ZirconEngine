use super::*;

#[test]
fn record_build_keeps_newest_history_and_last_success() {
    let mut engine = SourceEngineInstall::default();

    for index in 0..10 {
        engine.record_build(SourceBuildRecord {
            finished_unix_ms: index,
            status: if index == 9 { "success" } else { "failed" }.to_string(),
            profile: "debug".to_string(),
            jobs: Some(1),
            output_dir: PathBuf::from("E:/out"),
            detail: HubMessage::raw_text(format!("run {index}")),
            log_excerpt: HubMessage::raw_text(format!("log {index}")),
            command_line: vec!["python".to_string(), "tools/zircon_build.py".to_string()],
        });
    }

    assert_eq!(engine.last_build_unix_ms, Some(9));
    assert_eq!(engine.build_history.len(), BUILD_HISTORY_LIMIT);
    assert_eq!(engine.build_history[0].detail, "run 9");
    assert_eq!(engine.build_history[7].detail, "run 2");
}
