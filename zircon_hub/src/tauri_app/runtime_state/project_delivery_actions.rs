use std::path::PathBuf;

use crate::error::HubError;
use crate::projects::{
    install_package_to_device, package_project, validate_project_root, DeviceInstallReport,
    DeviceInstallRequest, ProjectPackageReport, ProjectPackageRequest, ProjectValidation,
    RecentProject,
};
use crate::state::{
    DeliveryMessageId, HubActionKind, HubActionRecord, HubActionStatus, HubMessage, HubMessageId,
    ProjectMessageId, TaskExecutionOutcome, TaskOperationKind, TaskStatus,
};
use crate::tauri_app::action_id::HubActionId;

use super::{
    action_tasks::{BackgroundTask, BackgroundTaskContext},
    recent_project_display_name, HubRuntimeSession,
};

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

    fn run(
        &self,
        context: &BackgroundTaskContext,
    ) -> Result<TaskExecutionOutcome<ProjectPackageReport>, HubError> {
        package_project(&self.request, context.cancellation())
    }
}

impl BackgroundTask for PendingDeviceInstall {
    type Output = (ProjectPackageReport, DeviceInstallReport);

    fn run(
        &self,
        context: &BackgroundTaskContext,
    ) -> Result<TaskExecutionOutcome<(ProjectPackageReport, DeviceInstallReport)>, HubError> {
        let package_report = match package_project(&self.package_request, context.cancellation())? {
            TaskExecutionOutcome::Completed(report) => report,
            TaskExecutionOutcome::Cancelled => return Ok(TaskExecutionOutcome::Cancelled),
        };
        let install_request =
            DeviceInstallRequest::new(package_report.package_dir.clone(), self.device_root.clone());
        resolve_device_install_stage(
            package_report,
            install_package_to_device(&install_request, context.cancellation()),
        )
    }
}

fn resolve_device_install_stage(
    package_report: ProjectPackageReport,
    install_result: Result<TaskExecutionOutcome<DeviceInstallReport>, HubError>,
) -> Result<TaskExecutionOutcome<(ProjectPackageReport, DeviceInstallReport)>, HubError> {
    match install_result {
        Ok(TaskExecutionOutcome::Completed(install_report)) => Ok(TaskExecutionOutcome::Completed(
            (package_report, install_report),
        )),
        Ok(TaskExecutionOutcome::Cancelled) => {
            package_report.remove_owned_output()?;
            Ok(TaskExecutionOutcome::Cancelled)
        }
        Err(error) => match package_report.remove_owned_output() {
            Ok(()) => Err(error),
            Err(cleanup_error) => Err(HubError::message(format!("{error}; {cleanup_error}"))),
        },
    }
}

impl HubRuntimeSession {
    pub(in crate::tauri_app) fn prepare_background_project_package(
        &mut self,
    ) -> Result<Option<PendingProjectPackage>, HubError> {
        let history_len = self.config.action_history.len();
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
            Err(error) if self.config.action_history.len() == history_len => Err(error),
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
        let history_len = self.config.action_history.len();
        match self.prepare_device_install() {
            Ok(pending_install) => {
                self.mark_background_action_prepared();
                Ok(Some(pending_install))
            }
            Err(error) if self.config.action_history.len() == history_len => Err(error),
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
            report.receipt_path.to_string_lossy().into_owned(),
            report.files_copied.to_string(),
        ],
    )
}

#[cfg(test)]
#[path = "tests/project_delivery_actions.rs"]
mod tests;
