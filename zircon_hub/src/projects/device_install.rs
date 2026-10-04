use std::fs;
use std::path::{Path, PathBuf};

use crate::error::HubError;
use crate::state::{
    DeliveryMessageId, HubMessage, HubMessageId, TaskCancellationToken, TaskExecutionOutcome,
};

use super::install_receipt::write_install_receipt;
use super::local_paths::{
    cleanup_dir_on_error, create_owned_dir, reject_inside_root, remove_owned_dir,
};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DeviceInstallRequest {
    pub package_dir: PathBuf,
    pub device_root: PathBuf,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DeviceInstallReport {
    pub install_dir: PathBuf,
    pub receipt_path: PathBuf,
    pub files_copied: usize,
    pub total_bytes: u64,
}

impl DeviceInstallRequest {
    pub fn new(package_dir: impl Into<PathBuf>, device_root: impl Into<PathBuf>) -> Self {
        Self {
            package_dir: package_dir.into(),
            device_root: device_root.into(),
        }
    }
}

pub fn install_package_to_device(
    request: &DeviceInstallRequest,
    cancellation: &TaskCancellationToken,
) -> Result<TaskExecutionOutcome<DeviceInstallReport>, HubError> {
    if cancellation.is_cancellation_requested() {
        return Ok(TaskExecutionOutcome::Cancelled);
    }
    if request.package_dir.as_os_str().is_empty() || !request.package_dir.is_dir() {
        return Err(HubError::status(
            HubMessage::new(HubMessageId::Delivery(
                DeliveryMessageId::PackageDirectoryUnavailable,
            )),
            None,
        ));
    }
    if request.device_root.as_os_str().is_empty() {
        return Err(HubError::status(
            HubMessage::new(HubMessageId::Delivery(
                DeliveryMessageId::DeviceInstallRequired,
            )),
            None,
        ));
    }

    reject_device_inside_package(&request.package_dir, &request.device_root)?;
    fs::create_dir_all(&request.device_root)?;

    let install_dir = request
        .device_root
        .join(package_install_name(&request.package_dir));
    create_owned_dir(&install_dir, || {
        HubMessage::with_params(
            HubMessageId::Delivery(DeliveryMessageId::DeviceInstallAlreadyExists),
            [install_dir.to_string_lossy().into_owned()],
        )
    })?;
    match fill_install_dir(&request.package_dir, &install_dir, cancellation) {
        Ok(TaskExecutionOutcome::Completed(report)) => Ok(TaskExecutionOutcome::Completed(report)),
        Ok(TaskExecutionOutcome::Cancelled) => {
            remove_owned_dir(&install_dir)?;
            Ok(TaskExecutionOutcome::Cancelled)
        }
        Err(error) => cleanup_dir_on_error(&install_dir, Err(error)),
    }
}

fn fill_install_dir(
    package_dir: &Path,
    install_dir: &Path,
    cancellation: &TaskCancellationToken,
) -> Result<TaskExecutionOutcome<DeviceInstallReport>, HubError> {
    let files_copied = match copy_directory_tree(package_dir, install_dir, cancellation)? {
        TaskExecutionOutcome::Completed(files_copied) => files_copied,
        TaskExecutionOutcome::Cancelled => return Ok(TaskExecutionOutcome::Cancelled),
    };
    let (receipt_path, receipt) = match write_install_receipt(install_dir, cancellation)? {
        TaskExecutionOutcome::Completed(receipt) => receipt,
        TaskExecutionOutcome::Cancelled => return Ok(TaskExecutionOutcome::Cancelled),
    };
    Ok(TaskExecutionOutcome::Completed(DeviceInstallReport {
        install_dir: install_dir.to_path_buf(),
        receipt_path,
        files_copied,
        total_bytes: receipt.total_bytes,
    }))
}

fn reject_device_inside_package(package_dir: &Path, device_root: &Path) -> Result<(), HubError> {
    reject_inside_root(
        package_dir,
        device_root,
        HubMessage::new(HubMessageId::Delivery(
            DeliveryMessageId::DeviceInstallOutsidePackage,
        )),
    )
}

fn package_install_name(package_dir: &Path) -> String {
    package_dir
        .file_name()
        .and_then(|name| name.to_str())
        .filter(|name| !name.trim().is_empty())
        .unwrap_or("zircon-package")
        .to_string()
}

fn copy_directory_tree(
    source: &Path,
    destination: &Path,
    cancellation: &TaskCancellationToken,
) -> Result<TaskExecutionOutcome<usize>, HubError> {
    let mut files_copied = 0;
    for entry in fs::read_dir(source)? {
        if cancellation.is_cancellation_requested() {
            return Ok(TaskExecutionOutcome::Cancelled);
        }
        let entry = entry?;
        let source_path = entry.path();
        let target_path = destination.join(entry.file_name());
        let file_type = entry.file_type()?;

        if file_type.is_dir() {
            fs::create_dir_all(&target_path)?;
            match copy_directory_tree(&source_path, &target_path, cancellation)? {
                TaskExecutionOutcome::Completed(copied) => files_copied += copied,
                TaskExecutionOutcome::Cancelled => return Ok(TaskExecutionOutcome::Cancelled),
            }
        } else if file_type.is_file() {
            fs::copy(&source_path, &target_path)?;
            files_copied += 1;
        }
    }
    Ok(TaskExecutionOutcome::Completed(files_copied))
}

#[cfg(test)]
#[path = "tests/device_install.rs"]
mod tests;
