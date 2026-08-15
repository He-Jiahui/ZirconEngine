use std::path::PathBuf;

use crate::error::HubError;
use crate::process::{open_folder, OpenFolderCommand};
use crate::state::{
    DeliveryMessageId, HubActionKind, HubActionRecord, HubActionStatus, HubMessage, HubMessageId,
    ShellMessageId, TaskOperationKind, TaskStatus,
};
use crate::tauri_app::action_request::OpenOutputFolderPayload;

use super::HubRuntimeSession;

impl HubRuntimeSession {
    pub(super) fn open_output_folder(
        &mut self,
        target_id: Option<&str>,
        payload: Option<OpenOutputFolderPayload>,
    ) -> Result<(), HubError> {
        let output_dir = match self.resolve_output_folder(target_id, payload) {
            Ok(output_dir) => output_dir,
            Err(error) => {
                let (detail, recovery) = error.into_status_messages();
                self.record_output_folder_failure(
                    "Output Folder".to_string(),
                    detail,
                    recovery.unwrap_or_else(|| {
                        HubMessage::new(HubMessageId::Delivery(
                            DeliveryMessageId::ChooseRecordedOutputRecovery,
                        ))
                    }),
                )?;
                return Ok(());
            }
        };
        if !output_dir.is_dir() {
            self.record_output_folder_failure(
                output_dir.to_string_lossy().into_owned(),
                HubMessage::with_params(
                    HubMessageId::Delivery(DeliveryMessageId::OutputFolderDoesNotExist),
                    [output_dir.to_string_lossy().into_owned()],
                ),
                HubMessage::new(HubMessageId::Delivery(
                    DeliveryMessageId::RunWorkflowAgainRecovery,
                )),
            )?;
            return Ok(());
        }

        let command = OpenFolderCommand::new(output_dir.clone());
        let command_line = command.command_line();
        match open_folder(&command) {
            Ok(child) => {
                let process_id = child.id();
                crate::state::push_action_record(
                    &mut self.config.action_history,
                    HubActionRecord {
                        finished_unix_ms: crate::projects::now_unix_ms(),
                        action: HubActionKind::OpenOutput,
                        status: HubActionStatus::Success,
                        target: output_dir.to_string_lossy().into_owned(),
                        detail: HubMessage::with_params(
                            HubMessageId::Shell(ShellMessageId::OpenedPath),
                            [output_dir.to_string_lossy().into_owned()],
                        ),
                        log_excerpt: HubMessage::empty(),
                        recovery: None,
                        process_id: Some(process_id),
                        command_line,
                        output_dir: Some(output_dir.clone()),
                    },
                );
                self.task_status = TaskStatus::success(
                    "Output folder opened",
                    HubMessage::raw_text(output_dir.to_string_lossy().into_owned()),
                )
                .with_operation(TaskOperationKind::Process, output_dir.to_string_lossy());
                self.persist()
            }
            Err(error) => self.record_output_folder_failure(
                output_dir.to_string_lossy().into_owned(),
                HubMessage::raw_text(error.to_string()),
                HubMessage::new(HubMessageId::Delivery(
                    DeliveryMessageId::OpenFolderManuallyRecovery,
                )),
            ),
        }
    }

    fn resolve_output_folder(
        &self,
        target_id: Option<&str>,
        payload: Option<OpenOutputFolderPayload>,
    ) -> Result<PathBuf, HubError> {
        if let Some(payload) = payload.as_ref() {
            if let Some(output_dir) = payload.output_dir.clone() {
                return Ok(output_dir);
            }
            if let Some(path) = payload.path.clone() {
                return Ok(path);
            }
        }

        let target = payload.and_then(|payload| payload.history_id).or_else(|| {
            target_id
                .map(str::trim)
                .filter(|target| !target.is_empty())
                .map(str::to_string)
        });

        let Some(target) = target else {
            return Err(HubError::status(
                HubMessage::new(HubMessageId::Delivery(
                    DeliveryMessageId::OpenOutputTargetRequired,
                )),
                Some(HubMessage::new(HubMessageId::Delivery(
                    DeliveryMessageId::ChooseRecordedOutputRecovery,
                ))),
            ));
        };

        if let Some(record) = self.config.action_history.iter().find(|record| {
            action_history_id(record) == target
                || record.target == target
                || record.detail == target
        }) {
            if let Some(output_dir) = record.output_dir.clone() {
                return Ok(output_dir);
            }
        }

        Ok(PathBuf::from(target))
    }

    fn record_output_folder_failure(
        &mut self,
        target: String,
        detail: HubMessage,
        recovery: HubMessage,
    ) -> Result<(), HubError> {
        crate::state::push_action_record(
            &mut self.config.action_history,
            HubActionRecord {
                finished_unix_ms: crate::projects::now_unix_ms(),
                action: HubActionKind::OpenOutput,
                status: HubActionStatus::Failed,
                target: target.clone(),
                detail: detail.clone(),
                log_excerpt: detail.clone(),
                recovery: Some(recovery.clone()),
                process_id: None,
                command_line: Vec::new(),
                output_dir: None,
            },
        );
        self.task_status = TaskStatus::error("Open Output failed", detail, recovery)
            .with_operation(TaskOperationKind::Process, target);
        self.persist()
    }
}

fn action_history_id(record: &HubActionRecord) -> String {
    format!(
        "{}:{}:{}",
        record.finished_unix_ms,
        record.action.id(),
        record.target
    )
}

#[cfg(test)]
mod tests {
    use std::{fs, path::PathBuf};

    use crate::{
        settings::{HubConfig, HubLanguage},
        state::{HubActionKind, HubActionRecord, HubActionStatus, HubMessage},
    };

    use super::super::{HubActionRequest, HubRuntimeSession};
    use crate::tauri_app::action_request::HubAction;

    #[test]
    fn open_output_folder_payload_accepts_flat_output_path() {
        let temp = temp_test_dir("zircon-hub-open-output-payload");
        let output_dir = temp.join("package-output");

        let action = HubActionRequest {
            action_id: "open-output-folder".to_string(),
            target_id: None,
            payload: Some(serde_json::json!({
                "path": output_dir.to_string_lossy()
            })),
        }
        .parse()
        .expect("output folder payload should parse");
        let HubAction::OpenOutputFolder { payload, .. } = action else {
            panic!("open-output-folder should parse to open-output action");
        };
        let path = payload
            .expect("output folder payload should be present")
            .path
            .expect("output folder path should be present");

        assert_eq!(path, output_dir);
        fs::remove_dir_all(temp).unwrap();
    }

    #[test]
    fn open_output_folder_resolves_record_id_before_path_fallback() {
        let temp = temp_test_dir("zircon-hub-open-output-history");
        let output_dir = temp.join("package-output");
        let session = session_with_output_history(&temp, &output_dir);
        let history_id = super::action_history_id(&session.config.action_history[0]);

        let resolved = session
            .resolve_output_folder(Some(&history_id), None)
            .expect("history id should resolve to recorded output dir");

        assert_eq!(resolved, output_dir);
        fs::remove_dir_all(temp).unwrap();
    }

    #[test]
    fn open_output_folder_prefers_typed_output_dir_over_archived_path_payload() {
        let temp = temp_test_dir("zircon-hub-open-output-typed-output-dir");
        let stale_path = temp.join("stale-visible-row-path");
        let output_dir = temp.join("package-output");
        let session = session_with_output_history(&temp, &output_dir);

        let resolved = session
            .resolve_output_folder(
                None,
                Some(crate::tauri_app::action_request::OpenOutputFolderPayload {
                    path: Some(stale_path),
                    output_dir: Some(output_dir.clone()),
                    history_id: None,
                }),
            )
            .expect("typed outputDir should resolve before archived path");

        assert_eq!(resolved, output_dir);
        fs::remove_dir_all(temp).unwrap();
    }

    #[test]
    fn open_output_folder_missing_directory_is_recoverable_status() {
        let temp = temp_test_dir("zircon-hub-open-output-missing");
        let output_dir = temp.join("missing-output");
        let mut session = session_with_output_history(&temp, &output_dir);

        session
            .apply_action(HubActionRequest {
                action_id: "open-output-folder".to_string(),
                target_id: Some(output_dir.to_string_lossy().into_owned()),
                payload: None,
            })
            .expect("missing output folder should be a recoverable Hub error");

        assert_eq!(session.task_status.label, "Open Output failed");
        assert_eq!(
            session.config.action_history[0].status,
            HubActionStatus::Failed
        );
        assert_eq!(
            session.config.action_history[0].action,
            HubActionKind::OpenOutput
        );

        fs::remove_dir_all(temp).unwrap();
    }

    #[test]
    fn open_output_folder_missing_target_failure_localizes_task_summary() {
        let temp = temp_test_dir("zircon-hub-open-output-missing-target-localized");
        let output_dir = temp.join("package-output");
        let mut session = session_with_output_history(&temp, &output_dir);
        session.config.settings.language = HubLanguage::Chinese;

        let view_model = session
            .apply_action(HubActionRequest {
                action_id: "open-output-folder".to_string(),
                target_id: None,
                payload: None,
            })
            .expect("missing output target should be a recoverable Hub error");

        assert_eq!(view_model.task_summary.label, "打开输出失败");
        assert_eq!(view_model.task_summary.detail, "需要打开输出目标");
        assert_eq!(
            view_model.task_summary.recovery.as_deref(),
            Some("打开文件夹前先选择已记录的包、安装或构建输出")
        );

        fs::remove_dir_all(temp).unwrap();
    }

    #[test]
    fn open_output_folder_missing_directory_failure_localizes_task_summary() {
        let temp = temp_test_dir("zircon-hub-open-output-missing-localized");
        let output_dir = temp.join("missing-output");
        let mut session = session_with_output_history(&temp, &output_dir);
        session.config.settings.language = HubLanguage::Chinese;

        let view_model = session
            .apply_action(HubActionRequest {
                action_id: "open-output-folder".to_string(),
                target_id: Some(output_dir.to_string_lossy().into_owned()),
                payload: None,
            })
            .expect("missing output folder should localize the recoverable Hub error");

        assert_eq!(view_model.task_summary.label, "打开输出失败");
        assert_eq!(
            view_model.task_summary.detail,
            format!("输出文件夹不存在：{}", output_dir.to_string_lossy())
        );
        assert_eq!(
            view_model.task_summary.recovery.as_deref(),
            Some("重新运行构建、打包或安装工作流后再打开输出文件夹")
        );

        fs::remove_dir_all(temp).unwrap();
    }

    fn session_with_output_history(
        temp: &std::path::Path,
        output_dir: &std::path::Path,
    ) -> HubRuntimeSession {
        let config_path = temp.join("hub.toml");
        let shared_recent_projects_path = temp.join("recent_projects.json");
        let mut config = HubConfig::default();
        config.action_history.push(HubActionRecord {
            finished_unix_ms: 42,
            action: HubActionKind::PackageProject,
            status: HubActionStatus::Success,
            target: "Game".to_string(),
            detail: HubMessage::raw_text("Packaged Game"),
            log_excerpt: HubMessage::empty(),
            recovery: None,
            process_id: None,
            command_line: Vec::new(),
            output_dir: Some(output_dir.to_path_buf()),
        });
        config.save(&config_path).unwrap();
        fs::write(
            &shared_recent_projects_path,
            r#"{"protocol_version":1,"projects":[]}"#,
        )
        .unwrap();
        HubRuntimeSession::load_from_paths(config_path, shared_recent_projects_path).unwrap()
    }

    fn temp_test_dir(prefix: &str) -> PathBuf {
        let path = std::env::temp_dir().join(format!(
            "{prefix}-{}-{}",
            std::process::id(),
            crate::projects::now_unix_ms()
        ));
        let _ = fs::remove_dir_all(&path);
        fs::create_dir_all(&path).unwrap();
        path
    }
}
