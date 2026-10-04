use std::path::PathBuf;

use crate::engines::{SourceBuildRecord, SourceEngineInstall};
use crate::settings::HubLanguage;
use crate::state::{EngineMessageId, HubMessage, HubMessageId};

#[test]
fn source_build_history_rows_localize_detail_status_and_finished_time() {
    let engine = SourceEngineInstall {
        id: "source-local".to_string(),
        display_name: "Local Source".to_string(),
        source_dir: PathBuf::from("E:/Source/ZirconEngine"),
        output_dir: PathBuf::from("E:/Source/ZirconEngine/out"),
        last_build_unix_ms: Some(1_000),
        build_history: vec![SourceBuildRecord {
            finished_unix_ms: 1_000,
            status: "success".to_string(),
            profile: "debug".to_string(),
            jobs: Some(4),
            output_dir: PathBuf::from("E:/Source/ZirconEngine/out"),
            detail: HubMessage::new(HubMessageId::Engine(
                EngineMessageId::StagedEditorRuntimePayload,
            )),
            log_excerpt: HubMessage::new(HubMessageId::Engine(
                EngineMessageId::StagedEditorRuntimePayload,
            )),
            command_line: vec!["python".to_string(), "tools/zircon_build.py".to_string()],
        }],
    };

    let rows = super::source_build_history_rows(&engine, 1_000, HubLanguage::Chinese);

    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0].status, "成功");
    assert_eq!(rows[0].status_tone, "success");
    assert_eq!(rows[0].detail, "已暂存编辑器/运行时载荷");
    assert_eq!(
        rows[0].secondary_detail,
        "命令：python tools/zircon_build.py；日志：已暂存编辑器/运行时载荷"
    );
    assert_eq!(rows[0].log_excerpt, "已暂存编辑器/运行时载荷");
    assert_eq!(rows[0].finished, "刚刚");
    assert_eq!(rows[0].output_dir, "E:/Source/ZirconEngine/out");
}
