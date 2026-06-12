use crate::error::HubError;
use crate::process::{open_folder, OpenFolderCommand};
use crate::state::{
    DeliveryMessageId, HubActionKind, HubActionRecord, HubActionStatus, HubMessage, HubMessageId,
    LearnMessageId, ShellMessageId, TaskOperationKind, TaskStatus,
};
use crate::tauri_app::action_request::OpenResourcePayload;

use super::HubRuntimeSession;

impl HubRuntimeSession {
    pub(super) fn open_learn_resource(
        &mut self,
        target_id: Option<&str>,
        payload: Option<OpenResourcePayload>,
    ) -> Result<(), HubError> {
        let targets = match open_resource_targets(target_id, payload) {
            Ok(targets) => targets,
            Err(error) => {
                let (detail, recovery) = error.into_status_messages();
                self.record_open_resource_failure(
                    "Learn Resource".to_string(),
                    detail,
                    recovery.unwrap_or_else(|| {
                        HubMessage::new(HubMessageId::Learn(LearnMessageId::ChooseResource))
                    }),
                )?;
                return Ok(());
            }
        };
        let fallback_target = targets
            .first()
            .cloned()
            .unwrap_or_else(|| "Learn Resource".to_string());
        let Some(resource) = self.learn_catalog.iter().find(|resource| {
            let path = resource.path.to_string_lossy();
            targets
                .iter()
                .any(|target| path == target.as_str() || resource.title == target.as_str())
        }) else {
            self.record_open_resource_failure(
                fallback_target,
                HubMessage::new(HubMessageId::Learn(LearnMessageId::ResourceNotInCatalog)),
                HubMessage::new(HubMessageId::Learn(
                    LearnMessageId::RefreshOrChooseLocalDocument,
                )),
            )?;
            return Ok(());
        };
        if !resource.path.is_file() {
            self.record_open_resource_failure(
                resource.title.clone(),
                HubMessage::with_params(
                    HubMessageId::Learn(LearnMessageId::ResourceFileDoesNotExist),
                    [resource.path.to_string_lossy().into_owned()],
                ),
                HubMessage::new(HubMessageId::Learn(
                    LearnMessageId::RefreshAndChooseLocalDocument,
                )),
            )?;
            return Ok(());
        }
        let folder = resource
            .path
            .parent()
            .map(|path| path.to_path_buf())
            .unwrap_or_else(|| resource.path.clone());
        let command = OpenFolderCommand::new(folder.clone());
        let command_line = command.command_line();
        match open_folder(&command) {
            Ok(child) => {
                let process_id = child.id();
                crate::state::push_action_record(
                    &mut self.config.action_history,
                    HubActionRecord {
                        finished_unix_ms: crate::projects::now_unix_ms(),
                        action: HubActionKind::OpenResource,
                        status: HubActionStatus::Success,
                        target: resource.title.clone(),
                        detail: HubMessage::with_params(
                            HubMessageId::Shell(ShellMessageId::OpenedPath),
                            [resource.path.to_string_lossy().into_owned()],
                        ),
                        log_excerpt: HubMessage::empty(),
                        recovery: None,
                        process_id: Some(process_id),
                        command_line,
                        output_dir: Some(folder),
                    },
                );
                self.task_status = TaskStatus::success(
                    "Resource opened",
                    HubMessage::legacy(resource.path.to_string_lossy().into_owned()),
                )
                .with_operation(TaskOperationKind::Hub, resource.title.clone());
                self.persist(None)
            }
            Err(error) => self.record_open_resource_failure(
                resource.title.clone(),
                HubMessage::legacy(error.to_string()),
                HubMessage::new(HubMessageId::Delivery(
                    DeliveryMessageId::OpenContainingFolderRecovery,
                )),
            ),
        }
    }

    fn record_open_resource_failure(
        &mut self,
        target: String,
        detail: HubMessage,
        recovery: HubMessage,
    ) -> Result<(), HubError> {
        crate::state::push_action_record(
            &mut self.config.action_history,
            HubActionRecord {
                finished_unix_ms: crate::projects::now_unix_ms(),
                action: HubActionKind::OpenResource,
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
        self.task_status = TaskStatus::error("Open Resource failed", detail, recovery)
            .with_operation(TaskOperationKind::Hub, target);
        self.persist(None)
    }
}

fn open_resource_targets(
    target_id: Option<&str>,
    payload: Option<OpenResourcePayload>,
) -> Result<Vec<String>, HubError> {
    let mut targets = Vec::new();
    if let Some(target) = target_id.map(str::trim).filter(|target| !target.is_empty()) {
        push_unique_resource_target(&mut targets, target.to_string());
    }
    if let Some(payload) = payload {
        if let Some(resource_id) = payload
            .resource_id
            .filter(|target| !target.trim().is_empty())
        {
            push_unique_resource_target(&mut targets, resource_id.trim().to_string());
        }
        if let Some(path) = payload.path {
            let path = path.to_string_lossy().into_owned();
            if !path.trim().is_empty() {
                push_unique_resource_target(&mut targets, path);
            }
        }
    }

    if targets.is_empty() {
        Err(HubError::status(
            HubMessage::new(HubMessageId::Learn(
                LearnMessageId::OpenResourceTargetRequired,
            )),
            Some(HubMessage::new(HubMessageId::Learn(
                LearnMessageId::ChooseResource,
            ))),
        ))
    } else {
        Ok(targets)
    }
}

fn push_unique_resource_target(targets: &mut Vec<String>, target: String) {
    if !targets.iter().any(|existing| existing == &target) {
        targets.push(target);
    }
}

#[cfg(test)]
mod tests {
    use std::{fs, path::PathBuf};

    use crate::{settings::HubConfig, state::HubActionStatus};

    use super::super::HubRuntimeSession;
    use crate::tauri_app::{action_request::HubAction, HubActionRequest};

    #[test]
    fn open_resource_rejects_paths_outside_current_learn_catalog() {
        let temp = temp_test_dir("zircon-hub-open-resource-reject");
        let mut session = session_with_docs(&temp);

        session
            .apply_action(HubActionRequest {
                action_id: "open-resource".to_string(),
                target_id: Some(temp.join("outside.md").to_string_lossy().into_owned()),
                payload: None,
            })
            .expect("unknown resource should be a recoverable Hub error");

        assert_eq!(session.task_status.label, "Open Resource failed");
        assert_eq!(
            session.config.action_history[0].status,
            HubActionStatus::Failed
        );

        fs::remove_dir_all(temp).unwrap();
    }

    #[test]
    fn open_resource_payload_resolves_resource_path_before_catalog_check() {
        let temp = temp_test_dir("zircon-hub-open-resource-payload");
        let session = session_with_docs(&temp);
        let resource = session.learn_catalog[0].path.clone();

        let action = HubActionRequest {
            action_id: "open-resource".to_string(),
            target_id: None,
            payload: Some(serde_json::json!({
                "path": resource.to_string_lossy()
            })),
        }
        .parse()
        .expect("resource payload should parse");
        let HubAction::OpenResource { payload, .. } = action else {
            panic!("open-resource should parse to open-resource action");
        };
        let targets = super::open_resource_targets(None, payload)
            .expect("resource target should resolve from parsed payload");

        assert_eq!(targets, vec![resource.to_string_lossy().into_owned()]);
        fs::remove_dir_all(temp).unwrap();
    }

    #[test]
    fn open_resource_payload_path_can_identify_catalog_entry_when_resource_id_is_stale() {
        let temp = temp_test_dir("zircon-hub-open-resource-path-fallback");
        let mut session = session_with_docs(&temp);
        session.config.settings.language = crate::settings::HubLanguage::Chinese;
        let resource_path = session.learn_catalog[0].path.clone();
        fs::remove_file(&resource_path).unwrap();

        let model = session
            .apply_action(HubActionRequest {
                action_id: "open-resource".to_string(),
                target_id: None,
                payload: Some(serde_json::json!({
                    "resourceId": "stale-learn-row-id",
                    "path": resource_path.to_string_lossy()
                })),
            })
            .expect("open-resource should fall back to catalog path from typed payload");

        assert_eq!(model.task_summary.label, "打开资源失败");
        assert_eq!(
            model.task_summary.detail,
            format!("资源文件不存在：{}", resource_path.to_string_lossy())
        );
        assert_eq!(
            model.action_history[0].target,
            session.learn_catalog[0].title
        );

        fs::remove_dir_all(temp).unwrap();
    }

    #[test]
    fn open_resource_missing_catalog_file_failure_localizes_task_summary() {
        let temp = temp_test_dir("zircon-hub-open-resource-missing-file");
        let mut session = session_with_docs(&temp);
        session.config.settings.language = crate::settings::HubLanguage::Chinese;
        let resource_path = session
            .learn_catalog
            .iter()
            .find(|resource| resource.path.starts_with(&temp))
            .expect("temp Learn resource should be present in catalog")
            .path
            .clone();
        fs::remove_file(&resource_path).unwrap();

        session
            .apply_action(HubActionRequest {
                action_id: "open-resource".to_string(),
                target_id: Some(resource_path.to_string_lossy().into_owned()),
                payload: None,
            })
            .expect("missing catalog resource file should be recoverable");

        let model = session.view_model();

        assert_eq!(model.task_summary.label, "打开资源失败");
        assert_eq!(
            model.task_summary.detail,
            format!("资源文件不存在：{}", resource_path.to_string_lossy())
        );
        assert_eq!(
            model.task_summary.recovery.as_deref(),
            Some("刷新学习目录并选择已有本地文档")
        );
        assert_eq!(
            model.action_history[0].detail,
            format!("资源文件不存在：{}", resource_path.to_string_lossy())
        );
        assert_eq!(
            model.action_history[0].recovery.as_deref(),
            Some("刷新学习目录并选择已有本地文档")
        );

        fs::remove_dir_all(temp).unwrap();
    }

    #[test]
    fn open_resource_missing_target_failure_localizes_task_summary_and_history() {
        let temp = temp_test_dir("zircon-hub-open-resource-missing-target");
        let mut session = session_with_docs(&temp);
        session.config.settings.language = crate::settings::HubLanguage::Chinese;

        let model = session
            .apply_action(HubActionRequest {
                action_id: "open-resource".to_string(),
                target_id: None,
                payload: None,
            })
            .expect("missing resource target should be recoverable");

        assert_eq!(model.task_summary.label, "打开资源失败");
        assert_eq!(model.task_summary.detail, "需要打开资源目标");
        assert_eq!(
            model.task_summary.recovery.as_deref(),
            Some("从当前学习目录中选择资源")
        );
        assert_eq!(model.action_history[0].detail, "需要打开资源目标");
        assert_eq!(
            model.action_history[0].recovery.as_deref(),
            Some("从当前学习目录中选择资源")
        );

        fs::remove_dir_all(temp).unwrap();
    }

    fn session_with_docs(temp: &std::path::Path) -> HubRuntimeSession {
        let source = temp.join("ZirconEngine");
        fs::create_dir_all(source.join("docs")).unwrap();
        fs::create_dir_all(source.join("tools")).unwrap();
        fs::create_dir_all(source.join("zircon_runtime")).unwrap();
        fs::write(
            source.join("Cargo.toml"),
            "[workspace]\nmembers = [\"zircon_runtime\"]\n",
        )
        .unwrap();
        fs::write(source.join("tools").join("zircon_build.py"), "").unwrap();
        fs::write(
            source.join("docs").join("guide.md"),
            "# Guide\n\nLocal guide.",
        )
        .unwrap();
        let config_path = temp.join("hub.toml");
        let editor_config_path = temp.join("editor.json");
        let mut config = HubConfig::default();
        config.settings.default_source_dir = source;
        config.settings.default_build_output_dir = temp.join("out");
        config.save(&config_path).unwrap();
        fs::write(
            &editor_config_path,
            r#"{"editor.startup.session":{"recent_projects":[]}}"#,
        )
        .unwrap();
        HubRuntimeSession::load_from_paths(config_path, editor_config_path).unwrap()
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
