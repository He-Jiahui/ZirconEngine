use std::path::PathBuf;

use crate::error::HubError;
use crate::projects::{
    install_package_to_device, package_project, validate_project_root, DeviceInstallReport,
    DeviceInstallRequest, ProjectPackageReport, ProjectPackageRequest, ProjectValidation,
    RecentProject,
};
use crate::state::{
    DeliveryMessageId, HubActionKind, HubActionRecord, HubActionStatus, HubMessage, HubMessageId,
    ProjectMessageId, TaskOperationKind, TaskStatus,
};
use crate::tauri_app::action_id::HubActionId;

use super::{action_tasks::BackgroundTask, recent_project_display_name, HubRuntimeSession};

#[derive(Clone, Debug)]
pub(in crate::tauri_app) struct PendingProjectPackage {
    project_name: String,
    request: ProjectPackageRequest,
}

#[derive(Clone, Debug)]
pub(in crate::tauri_app) struct PendingDeviceInstall {
    project_name: String,
    package_request: ProjectPackageRequest,
    device_root: PathBuf,
}

impl BackgroundTask for PendingProjectPackage {
    type Output = ProjectPackageReport;

    fn run(&self) -> Result<ProjectPackageReport, HubError> {
        package_project(&self.request)
    }
}

impl BackgroundTask for PendingDeviceInstall {
    type Output = (ProjectPackageReport, DeviceInstallReport);

    fn run(&self) -> Result<(ProjectPackageReport, DeviceInstallReport), HubError> {
        let package_report = package_project(&self.package_request)?;
        let install_request =
            DeviceInstallRequest::new(package_report.package_dir.clone(), self.device_root.clone());
        let install_report = install_package_to_device(&install_request)?;
        Ok((package_report, install_report))
    }
}

impl HubRuntimeSession {
    pub(super) fn package_recent_project(&mut self) -> Result<(), HubError> {
        let pending_package = match self.prepare_project_package(
            HubMessage::new(HubMessageId::Project(
                ProjectMessageId::NoRecentProjectToPackage,
            )),
            HubMessage::new(HubMessageId::Project(
                ProjectMessageId::SelectedProjectStaleForPackage,
            )),
        ) {
            Ok(pending_package) => pending_package,
            Err(_) => return Ok(()),
        };
        let result = pending_package.run();
        self.complete_project_package(pending_package, result)
    }

    pub(super) fn install_recent_project_to_device(&mut self) -> Result<(), HubError> {
        let pending_install = match self.prepare_device_install() {
            Ok(pending_install) => pending_install,
            Err(_) => return Ok(()),
        };
        let result = pending_install.run();
        self.complete_device_install(pending_install, result)
    }

    pub(in crate::tauri_app) fn prepare_background_project_package(
        &mut self,
    ) -> Result<Option<PendingProjectPackage>, HubError> {
        match self.prepare_project_package(
            HubMessage::new(HubMessageId::Project(
                ProjectMessageId::NoRecentProjectToPackage,
            )),
            HubMessage::new(HubMessageId::Project(
                ProjectMessageId::SelectedProjectStaleForPackage,
            )),
        ) {
            Ok(pending_package) => {
                self.mark_background_action_prepared();
                Ok(Some(pending_package))
            }
            Err(error) if self.task_status.running => Err(error),
            Err(_) => Ok(None),
        }
    }

    pub(in crate::tauri_app) fn complete_background_project_package(
        &mut self,
        pending_package: PendingProjectPackage,
        result: Result<ProjectPackageReport, HubError>,
    ) -> Result<(), HubError> {
        self.complete_project_package(pending_package, result)
    }

    pub(in crate::tauri_app) fn prepare_background_device_install(
        &mut self,
    ) -> Result<Option<PendingDeviceInstall>, HubError> {
        match self.prepare_device_install() {
            Ok(pending_install) => {
                self.mark_background_action_prepared();
                Ok(Some(pending_install))
            }
            Err(error) if self.task_status.running => Err(error),
            Err(_) => Ok(None),
        }
    }

    pub(in crate::tauri_app) fn complete_background_device_install(
        &mut self,
        pending_install: PendingDeviceInstall,
        result: Result<(ProjectPackageReport, DeviceInstallReport), HubError>,
    ) -> Result<(), HubError> {
        self.complete_device_install(pending_install, result)
    }

    fn prepare_project_package(
        &mut self,
        missing_project_message: HubMessage,
        stale_project_message: HubMessage,
    ) -> Result<PendingProjectPackage, HubError> {
        let project = match self.selected_or_latest_recent_project_for_named_action(
            missing_project_message,
            stale_project_message,
        ) {
            Ok(project) => project,
            Err(error) => {
                let (detail, _) = error.into_status_messages();
                let recovery = HubMessage::new(HubMessageId::Project(
                    ProjectMessageId::SelectProjectBeforePackaging,
                ));
                self.record_project_action_failure(
                    HubActionKind::PackageProject,
                    self.action_target_for_project_failure(),
                    detail.clone(),
                    recovery.clone(),
                    Some(self.config.settings.default_build_output_dir.clone()),
                )?;
                return Err(HubError::status(detail, Some(recovery)));
            }
        };
        self.pending_project_package_from_project(project)
    }

    fn prepare_device_install(&mut self) -> Result<PendingDeviceInstall, HubError> {
        let package = match self.prepare_project_package(
            HubMessage::new(HubMessageId::Project(
                ProjectMessageId::NoRecentProjectToInstall,
            )),
            HubMessage::new(HubMessageId::Project(
                ProjectMessageId::SelectedProjectStaleForInstall,
            )),
        ) {
            Ok(package) => package,
            Err(error) => {
                let (detail, _) = error.into_status_messages();
                let recovery = HubMessage::new(HubMessageId::Project(
                    ProjectMessageId::SelectProjectBeforeInstalling,
                ));
                self.record_project_action_failure(
                    HubActionKind::InstallProject,
                    self.action_target_for_project_failure(),
                    detail.clone(),
                    recovery.clone(),
                    Some(self.config.settings.default_device_install_dir.clone()),
                )?;
                return Err(HubError::status(detail, Some(recovery)));
            }
        };
        Ok(PendingDeviceInstall {
            project_name: package.project_name,
            package_request: package.request,
            device_root: self.config.settings.default_device_install_dir.clone(),
        })
    }

    fn pending_project_package_from_project(
        &mut self,
        project: RecentProject,
    ) -> Result<PendingProjectPackage, HubError> {
        if validate_project_root(&project.path) != ProjectValidation::Valid {
            let detail = HubMessage::with_params(
                HubMessageId::Project(ProjectMessageId::RootInvalid),
                [project.path.to_string_lossy().into_owned()],
            );
            let recovery = HubMessage::new(HubMessageId::Project(
                ProjectMessageId::CheckProjectManifest,
            ));
            self.record_project_action_failure(
                HubActionKind::PackageProject,
                recent_project_display_name(&project),
                detail.clone(),
                recovery.clone(),
                Some(self.config.settings.default_build_output_dir.clone()),
            )?;
            return Err(HubError::status(detail, Some(recovery)));
        }
        let display_name = recent_project_display_name(&project);
        let request = ProjectPackageRequest::new(
            display_name.clone(),
            project.path.clone(),
            self.config.settings.default_build_output_dir.clone(),
        );
        Ok(PendingProjectPackage {
            project_name: display_name,
            request,
        })
    }

    fn complete_project_package(
        &mut self,
        pending_package: PendingProjectPackage,
        result: Result<ProjectPackageReport, HubError>,
    ) -> Result<(), HubError> {
        let PendingProjectPackage {
            project_name,
            request,
        } = pending_package;
        let report = match result {
            Ok(report) => report,
            Err(error) => {
                let (detail, _) = error.into_status_messages();
                self.record_project_action_failure(
                    HubActionKind::PackageProject,
                    project_name,
                    detail,
                    HubMessage::new(HubMessageId::Delivery(
                        DeliveryMessageId::CheckPackageOutputRecovery,
                    )),
                    Some(self.config.settings.default_build_output_dir.clone()),
                )?;
                return Ok(());
            }
        };
        let detail = self.record_package_success(project_name.clone(), &request, &report)?;
        self.task_status = TaskStatus::success("Package created", detail)
            .with_operation(TaskOperationKind::Project, project_name);
        Ok(())
    }

    fn complete_device_install(
        &mut self,
        pending_install: PendingDeviceInstall,
        result: Result<(ProjectPackageReport, DeviceInstallReport), HubError>,
    ) -> Result<(), HubError> {
        let PendingDeviceInstall {
            project_name,
            package_request,
            device_root,
        } = pending_install;
        let (package_report, install_report) = match result {
            Ok(reports) => reports,
            Err(error) => {
                let (detail, _) = error.into_status_messages();
                self.record_project_action_failure(
                    HubActionKind::InstallProject,
                    project_name,
                    detail,
                    HubMessage::new(HubMessageId::Delivery(
                        DeliveryMessageId::CheckInstallOutputRecovery,
                    )),
                    Some(self.config.settings.default_device_install_dir.clone()),
                )?;
                return Ok(());
            }
        };
        self.record_package_success(project_name.clone(), &package_request, &package_report)?;
        let detail = delivery_file_count_detail(
            &project_name,
            install_report.install_dir.to_string_lossy().as_ref(),
            install_report.files_copied,
        );
        let command_line = install_command_line(&package_request, &device_root);
        let log_excerpt = install_log_excerpt(&project_name, &install_report);
        self.record_action_and_persist(HubActionRecord {
            finished_unix_ms: crate::projects::now_unix_ms(),
            action: HubActionKind::InstallProject,
            status: HubActionStatus::Success,
            target: project_name.clone(),
            detail: detail.clone(),
            log_excerpt,
            recovery: None,
            process_id: None,
            command_line,
            output_dir: Some(install_report.install_dir.clone()),
        })?;
        self.task_status = TaskStatus::success("Installed to device", detail)
            .with_operation(TaskOperationKind::Project, project_name);
        Ok(())
    }

    fn record_package_success(
        &mut self,
        project_name: String,
        request: &ProjectPackageRequest,
        report: &ProjectPackageReport,
    ) -> Result<HubMessage, HubError> {
        let detail = delivery_file_count_detail(
            &project_name,
            report.package_dir.to_string_lossy().as_ref(),
            report.files_copied,
        );
        let log_excerpt = package_log_excerpt(&project_name, report);
        self.record_action_and_persist(HubActionRecord {
            finished_unix_ms: crate::projects::now_unix_ms(),
            action: HubActionKind::PackageProject,
            status: HubActionStatus::Success,
            target: project_name,
            detail: detail.clone(),
            log_excerpt,
            recovery: None,
            process_id: None,
            command_line: package_command_line(request),
            output_dir: Some(report.package_dir.clone()),
        })?;
        Ok(detail)
    }

    fn record_project_action_failure(
        &mut self,
        action: HubActionKind,
        target: String,
        detail: HubMessage,
        recovery: HubMessage,
        output_dir: Option<PathBuf>,
    ) -> Result<(), HubError> {
        self.record_action_and_persist(HubActionRecord {
            finished_unix_ms: crate::projects::now_unix_ms(),
            action,
            status: HubActionStatus::Failed,
            target: target.clone(),
            detail: detail.clone(),
            log_excerpt: HubMessage::empty(),
            recovery: Some(recovery.clone()),
            process_id: None,
            command_line: Vec::new(),
            output_dir,
        })?;
        self.set_action_failure_status(action, target, detail, recovery);
        Ok(())
    }
}

fn package_command_line(request: &ProjectPackageRequest) -> Vec<String> {
    vec![
        "zircon_hub".to_string(),
        HubActionId::PackageProject.as_str().to_string(),
        "--project".to_string(),
        request.project_root.to_string_lossy().into_owned(),
        "--output".to_string(),
        request.output_root.to_string_lossy().into_owned(),
    ]
}

fn install_command_line(
    package_request: &ProjectPackageRequest,
    device_root: &PathBuf,
) -> Vec<String> {
    vec![
        "zircon_hub".to_string(),
        HubActionId::InstallDevice.as_str().to_string(),
        "--project".to_string(),
        package_request.project_root.to_string_lossy().into_owned(),
        "--package-output".to_string(),
        package_request.output_root.to_string_lossy().into_owned(),
        "--device".to_string(),
        device_root.to_string_lossy().into_owned(),
    ]
}

fn delivery_file_count_detail(
    project_name: &str,
    output_path: &str,
    file_count: usize,
) -> HubMessage {
    HubMessage::with_params(
        HubMessageId::Delivery(DeliveryMessageId::FileCountDetail),
        [
            project_name.to_string(),
            output_path.to_string(),
            file_count.to_string(),
        ],
    )
}

fn package_log_excerpt(project_name: &str, report: &ProjectPackageReport) -> HubMessage {
    HubMessage::with_params(
        HubMessageId::Delivery(DeliveryMessageId::PackageLogExcerpt),
        [
            project_name.to_string(),
            report.package_dir.to_string_lossy().into_owned(),
            report.files_copied.to_string(),
        ],
    )
}

fn install_log_excerpt(project_name: &str, report: &DeviceInstallReport) -> HubMessage {
    HubMessage::with_params(
        HubMessageId::Delivery(DeliveryMessageId::InstallLogExcerpt),
        [
            project_name.to_string(),
            report.install_dir.to_string_lossy().into_owned(),
            report.files_copied.to_string(),
        ],
    )
}

#[cfg(test)]
mod tests {
    use std::{fs, path::PathBuf};

    use crate::projects::RecentProject;
    use crate::settings::{HubConfig, HubLanguage};
    use crate::state::{HubActionKind, HubActionStatus};

    use super::{super::HubRuntimeSession, BackgroundTask, PendingDeviceInstall};

    #[test]
    fn background_package_prepares_request_without_copying_or_recording_history() {
        let temp = temp_test_dir("zircon-hub-background-package-prepare");
        let project = create_project_root(&temp, "Game");
        let mut session = session_with_project(&temp, "Game", &project);

        let pending = session
            .prepare_background_project_package()
            .expect("package preparation should not fail hard")
            .expect("valid project should prepare package request");

        assert_eq!(pending.project_name, "Game");
        assert_eq!(session.config.action_history.len(), 0);
        assert!(!session.config.settings.default_build_output_dir.exists());

        fs::remove_dir_all(temp).unwrap();
    }

    #[test]
    fn background_package_completion_records_success_after_copy_result() {
        let temp = temp_test_dir("zircon-hub-background-package-complete");
        let project = create_project_root(&temp, "Game");
        let mut session = session_with_project(&temp, "Game", &project);
        let pending = session
            .prepare_background_project_package()
            .unwrap()
            .unwrap();
        let result = pending.run();

        session
            .complete_background_project_package(pending, result)
            .expect("package completion should record state");

        let record = &session.config.action_history[0];
        assert_eq!(record.action, HubActionKind::PackageProject);
        assert_eq!(record.status, HubActionStatus::Success);
        assert_eq!(session.task_status.label, "Package created");
        assert!(record
            .output_dir
            .as_ref()
            .unwrap()
            .join("zircon-package.toml")
            .is_file());
        assert!(record
            .command_line
            .iter()
            .any(|part| part == "package-project"));
        assert!(record.command_line.iter().any(|part| part == "--project"));
        assert!(record
            .command_line
            .iter()
            .any(|part| part == &project.to_string_lossy()));
        assert!(record.log_excerpt.contains("2 files"));

        fs::remove_dir_all(temp).unwrap();
    }

    #[test]
    fn package_action_uses_explicit_target_project_instead_of_selected_project() {
        let temp = temp_test_dir("zircon-hub-package-explicit-target");
        let selected_project = create_project_root(&temp, "Selected");
        let target_project = create_project_root(&temp, "Target");
        let mut session = session_with_projects(
            &temp,
            &[
                ("Selected", selected_project.clone()),
                ("Target", target_project.clone()),
            ],
            &selected_project,
        );

        session
            .apply_action(super::super::HubActionRequest {
                action_id: "package-project".to_string(),
                target_id: Some(target_project.to_string_lossy().into_owned()),
                payload: None,
            })
            .expect("package action should accept a project target id");

        let record = &session.config.action_history[0];
        assert_eq!(record.action, HubActionKind::PackageProject);
        assert_eq!(record.status, HubActionStatus::Success);
        assert_eq!(record.target, "Target");
        let output_dir = record
            .output_dir
            .as_ref()
            .expect("target package should record output dir");
        assert!(output_dir
            .file_name()
            .and_then(|name| name.to_str())
            .is_some_and(|name| name.starts_with("target-")));
        assert!(fs::read_to_string(output_dir.join("zircon-package.toml"))
            .unwrap()
            .contains("package_name = \"Target\""));

        fs::remove_dir_all(temp).unwrap();
    }

    #[test]
    fn package_completion_localizes_success_summary_and_history() {
        let temp = temp_test_dir("zircon-hub-background-package-complete-localized");
        let project = create_project_root(&temp, "Game");
        let mut session = session_with_project(&temp, "Game", &project);
        session.config.settings.language = HubLanguage::Chinese;
        let pending = session
            .prepare_background_project_package()
            .unwrap()
            .unwrap();
        let result = pending.run();

        session
            .complete_background_project_package(pending, result)
            .expect("package completion should record localized state");

        let package_dir = session.config.action_history[0]
            .output_dir
            .as_ref()
            .expect("package history should keep output dir")
            .to_string_lossy()
            .into_owned();
        let expected_detail = format!("Game -> {package_dir}（2 个文件）");
        let model = session.view_model();
        assert_eq!(model.task_summary.label, "包已创建");
        assert_eq!(model.task_summary.detail, expected_detail);
        assert_eq!(model.action_history[0].action, "打包项目");
        assert_eq!(model.action_history[0].detail, expected_detail);
        assert_eq!(
            model.action_history[0].log_excerpt,
            format!("已打包 Game 到 {package_dir}（2 个文件）")
        );

        fs::remove_dir_all(temp).unwrap();
    }

    #[test]
    fn background_install_runs_package_then_device_copy_before_recording_history() {
        let temp = temp_test_dir("zircon-hub-background-install-complete");
        let project = create_project_root(&temp, "Game");
        let mut session = session_with_project(&temp, "Game", &project);
        let pending = session
            .prepare_background_device_install()
            .unwrap()
            .unwrap();
        let result = pending.run();

        session
            .complete_background_device_install(pending, result)
            .expect("install completion should record package and install state");

        let install = &session.config.action_history[0];
        let package = &session.config.action_history[1];
        assert_eq!(install.action, HubActionKind::InstallProject);
        assert_eq!(package.action, HubActionKind::PackageProject);
        assert_eq!(session.task_status.label, "Installed to device");
        assert!(install
            .output_dir
            .as_ref()
            .unwrap()
            .join("zircon-package.toml")
            .is_file());
        assert!(package
            .command_line
            .iter()
            .any(|part| part == "package-project"));
        assert!(package.log_excerpt.contains("2 files"));
        assert!(install
            .command_line
            .iter()
            .any(|part| part == "install-device"));
        assert!(install.command_line.iter().any(|part| part == "--device"));
        assert!(install.log_excerpt.contains("3 files"));

        fs::remove_dir_all(temp).unwrap();
    }

    #[test]
    fn install_completion_localizes_success_summary_and_history() {
        let temp = temp_test_dir("zircon-hub-background-install-complete-localized");
        let project = create_project_root(&temp, "Game");
        let mut session = session_with_project(&temp, "Game", &project);
        session.config.settings.language = HubLanguage::Chinese;
        let pending = session
            .prepare_background_device_install()
            .unwrap()
            .unwrap();
        let result = pending.run();

        session
            .complete_background_device_install(pending, result)
            .expect("install completion should record localized state");

        let install_dir = session.config.action_history[0]
            .output_dir
            .as_ref()
            .expect("install history should keep output dir")
            .to_string_lossy()
            .into_owned();
        let expected_detail = format!("Game -> {install_dir}（3 个文件）");
        let model = session.view_model();
        assert_eq!(model.task_summary.label, "已安装到设备");
        assert_eq!(model.task_summary.detail, expected_detail);
        assert_eq!(model.action_history[0].action, "安装到设备");
        assert_eq!(model.action_history[0].detail, expected_detail);
        assert_eq!(
            model.action_history[0].log_excerpt,
            format!("已安装 Game 到 {install_dir}（3 个文件）")
        );

        fs::remove_dir_all(temp).unwrap();
    }

    #[test]
    fn package_failure_localizes_required_output_root_summary_and_history() {
        let temp = temp_test_dir("zircon-hub-package-output-required-localized");
        let project = create_project_root(&temp, "Game");
        let mut session = session_with_project(&temp, "Game", &project);
        session.config.settings.language = HubLanguage::Chinese;
        session.config.settings.default_build_output_dir = PathBuf::new();
        let pending = session
            .prepare_background_project_package()
            .unwrap()
            .expect("valid project should still prepare a package request");
        let result = pending.run();

        session
            .complete_background_project_package(pending, result)
            .expect("package failure should record recoverable state");

        assert_eq!(
            session.task_status.detail,
            "Package output root is required"
        );
        let model = session.view_model();
        assert_eq!(model.task_summary.label, "打包项目失败");
        assert_eq!(model.task_summary.detail, "需要包输出根目录");
        assert_eq!(model.action_history[0].detail, "需要包输出根目录");
        assert_eq!(
            model.action_history[0].recovery.as_deref(),
            Some("检查项目根目录是否存在，并确保包输出目录位于项目外")
        );

        fs::remove_dir_all(temp).unwrap();
    }

    #[test]
    fn package_failure_localizes_output_inside_project_summary_and_history() {
        let temp = temp_test_dir("zircon-hub-package-output-inside-localized");
        let project = create_project_root(&temp, "Game");
        let mut session = session_with_project(&temp, "Game", &project);
        session.config.settings.language = HubLanguage::Chinese;
        session.config.settings.default_build_output_dir = project.join("build-output");
        let pending = session
            .prepare_background_project_package()
            .unwrap()
            .expect("valid project should still prepare a package request");
        let result = pending.run();

        session
            .complete_background_project_package(pending, result)
            .expect("package failure should record recoverable state");

        assert_eq!(
            session.task_status.detail,
            "Package output root must be outside the project directory"
        );
        let model = session.view_model();
        assert_eq!(model.task_summary.detail, "包输出根目录必须位于项目目录外");
        assert_eq!(
            model.action_history[0].detail,
            "包输出根目录必须位于项目目录外"
        );

        fs::remove_dir_all(temp).unwrap();
    }

    #[test]
    fn install_failure_localizes_required_device_root_summary_and_history() {
        let temp = temp_test_dir("zircon-hub-install-device-required-localized");
        let project = create_project_root(&temp, "Game");
        let mut session = session_with_project(&temp, "Game", &project);
        session.config.settings.language = HubLanguage::Chinese;
        session.config.settings.default_device_install_dir = PathBuf::new();
        let pending = session
            .prepare_background_device_install()
            .unwrap()
            .expect("valid project should still prepare an install request");
        let result = pending.run();

        session
            .complete_background_device_install(pending, result)
            .expect("install failure should record recoverable state");

        assert_eq!(
            session.task_status.detail,
            "Device install directory is required"
        );
        let model = session.view_model();
        assert_eq!(model.task_summary.label, "安装到设备失败");
        assert_eq!(model.task_summary.detail, "需要设备安装目录");
        assert_eq!(model.action_history[0].detail, "需要设备安装目录");
        assert_eq!(
            model.action_history[0].recovery.as_deref(),
            Some("重试前检查包输出和已配置的本地设备安装目录")
        );

        fs::remove_dir_all(temp).unwrap();
    }

    #[test]
    fn install_failure_localizes_device_root_inside_package_summary_and_history() {
        let temp = temp_test_dir("zircon-hub-install-device-inside-localized");
        let project = create_project_root(&temp, "Game");
        let mut session = session_with_project(&temp, "Game", &project);
        session.config.settings.language = HubLanguage::Chinese;
        let pending = session
            .prepare_background_device_install()
            .unwrap()
            .expect("valid project should still prepare an install request");
        let package_dir = pending
            .package_request
            .output_root
            .join("packages")
            .join(format!("game-{}", pending.package_request.created_unix_ms));
        let pending = PendingDeviceInstall {
            device_root: package_dir.join("device"),
            ..pending
        };
        let result = pending.run();

        session
            .complete_background_device_install(pending, result)
            .expect("install failure should record recoverable state");

        assert_eq!(
            session.task_status.detail,
            "Device install directory must be outside the package directory"
        );
        let model = session.view_model();
        assert_eq!(model.task_summary.detail, "设备安装目录必须位于包目录外");
        assert_eq!(
            model.action_history[0].detail,
            "设备安装目录必须位于包目录外"
        );

        fs::remove_dir_all(temp).unwrap();
    }

    #[test]
    fn install_failure_localizes_duplicate_install_directory_summary_and_history() {
        let temp = temp_test_dir("zircon-hub-install-duplicate-localized");
        let project = create_project_root(&temp, "Game");
        let mut session = session_with_project(&temp, "Game", &project);
        session.config.settings.language = HubLanguage::Chinese;
        let pending = session
            .prepare_background_device_install()
            .unwrap()
            .expect("valid project should still prepare an install request");
        let package_dir = pending
            .package_request
            .output_root
            .join("packages")
            .join(format!("game-{}", pending.package_request.created_unix_ms));
        let install_dir = pending.device_root.join(
            package_dir
                .file_name()
                .expect("package dir should have an install name"),
        );
        fs::create_dir_all(&install_dir).unwrap();
        let result = pending.run();

        session
            .complete_background_device_install(pending, result)
            .expect("install failure should record recoverable state");

        let expected_detail = format!(
            "Device install already exists: {}",
            install_dir.to_string_lossy()
        );
        assert_eq!(session.task_status.detail, expected_detail);
        let model = session.view_model();
        let expected_localized = format!("设备安装已存在：{}", install_dir.to_string_lossy());
        assert_eq!(model.task_summary.detail, expected_localized);
        assert_eq!(model.action_history[0].detail, expected_localized);

        fs::remove_dir_all(temp).unwrap();
    }

    fn session_with_project(
        temp: &std::path::Path,
        name: &str,
        project: &std::path::Path,
    ) -> HubRuntimeSession {
        let config_path = temp.join("hub.toml");
        let editor_config_path = temp.join("editor.json");
        let mut config = HubConfig::default();
        config.settings.default_build_output_dir = temp.join("out");
        config.settings.default_device_install_dir = temp.join("device");
        config.recent_projects = vec![RecentProject::new(name, project, 1)];
        config.runtime.selected_project_path = Some(project.to_path_buf());
        config.save(&config_path).unwrap();
        fs::write(
            &editor_config_path,
            r#"{"editor.startup.session":{"recent_projects":[]}}"#,
        )
        .unwrap();
        HubRuntimeSession::load_from_paths(config_path, editor_config_path).unwrap()
    }

    fn session_with_projects(
        temp: &std::path::Path,
        projects: &[(&str, PathBuf)],
        selected_project: &std::path::Path,
    ) -> HubRuntimeSession {
        let config_path = temp.join("hub.toml");
        let editor_config_path = temp.join("editor.json");
        let mut config = HubConfig::default();
        config.settings.default_build_output_dir = temp.join("out");
        config.settings.default_device_install_dir = temp.join("device");
        config.recent_projects = projects
            .iter()
            .map(|(name, path)| RecentProject::new(*name, path, 1))
            .collect();
        config.runtime.selected_project_path = Some(selected_project.to_path_buf());
        config.save(&config_path).unwrap();
        fs::write(
            &editor_config_path,
            r#"{"editor.startup.session":{"recent_projects":[]}}"#,
        )
        .unwrap();
        HubRuntimeSession::load_from_paths(config_path, editor_config_path).unwrap()
    }

    fn create_project_root(temp: &std::path::Path, name: &str) -> PathBuf {
        let project = temp.join(name);
        fs::create_dir_all(project.join("Assets")).unwrap();
        fs::write(
            project.join("zircon-project.toml"),
            format!("name = \"{name}\"\n"),
        )
        .unwrap();
        fs::write(project.join("Assets").join("mesh.txt"), "mesh").unwrap();
        project
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
