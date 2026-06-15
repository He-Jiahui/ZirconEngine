use std::fs;
use std::path::{Path, PathBuf};

use crate::error::HubError;
use crate::state::{DeliveryMessageId, HubMessage, HubMessageId};

use super::install_receipt::write_install_receipt;
use super::local_paths::{cleanup_dir_on_error, create_owned_dir, reject_inside_root};

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
) -> Result<DeviceInstallReport, HubError> {
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
    let result = fill_install_dir(&request.package_dir, &install_dir);

    cleanup_dir_on_error(&install_dir, result)
}

fn fill_install_dir(
    package_dir: &Path,
    install_dir: &Path,
) -> Result<DeviceInstallReport, HubError> {
    let files_copied = copy_directory_tree(package_dir, install_dir)?;
    let (receipt_path, receipt) = write_install_receipt(install_dir)?;
    Ok(DeviceInstallReport {
        install_dir: install_dir.to_path_buf(),
        receipt_path,
        files_copied,
        total_bytes: receipt.total_bytes,
    })
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

fn copy_directory_tree(source: &Path, destination: &Path) -> Result<usize, HubError> {
    let mut files_copied = 0;
    for entry in fs::read_dir(source)? {
        let entry = entry?;
        let source_path = entry.path();
        let target_path = destination.join(entry.file_name());
        let file_type = entry.file_type()?;

        if file_type.is_dir() {
            fs::create_dir_all(&target_path)?;
            files_copied += copy_directory_tree(&source_path, &target_path)?;
        } else if file_type.is_file() {
            fs::copy(&source_path, &target_path)?;
            files_copied += 1;
        }
    }
    Ok(files_copied)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn install_package_to_device_copies_package_folder() {
        let package = temp_dir("device-package");
        let device = temp_dir("device-root");
        fs::write(package.join("zircon-package.toml"), "files_copied = 1").unwrap();
        fs::create_dir_all(package.join("project")).unwrap();
        fs::write(
            package.join("project").join("zircon-project.toml"),
            "name='Demo'",
        )
        .unwrap();

        let report =
            install_package_to_device(&DeviceInstallRequest::new(package.clone(), device.clone()))
                .unwrap();

        assert!(report.install_dir.starts_with(&device));
        assert!(report.install_dir.join("zircon-package.toml").is_file());
        assert!(report
            .install_dir
            .join("project")
            .join("zircon-project.toml")
            .is_file());
        assert_eq!(report.files_copied, 2);
        assert!(report.receipt_path.is_file());
        assert!(report.total_bytes > 0);
        let receipt = fs::read_to_string(&report.receipt_path).unwrap();
        assert!(receipt.contains("\"content_download_manifest\""));
        assert!(receipt.contains("\"sha256\""));
        assert!(receipt.contains("\"project/zircon-project.toml\""));

        fs::remove_dir_all(package).unwrap();
        fs::remove_dir_all(device).unwrap();
    }

    #[test]
    fn install_package_to_device_rejects_device_root_inside_package() {
        let package = temp_dir("device-package-inside");
        let device = package.join("device");
        let error =
            install_package_to_device(&DeviceInstallRequest::new(&package, &device)).unwrap_err();
        fs::remove_dir_all(package).unwrap();

        assert!(error.to_string().contains("outside the package directory"));
    }

    #[test]
    fn install_package_to_device_rejects_missing_device_root_inside_package_without_creating_directory(
    ) {
        let package = temp_dir("device-package-inside-missing");
        let device = package.join("device").join("nested");

        let error =
            install_package_to_device(&DeviceInstallRequest::new(&package, &device)).unwrap_err();

        assert!(error.to_string().contains("outside the package directory"));
        assert!(
            !device.exists(),
            "rejected device install roots inside the package must not be created"
        );

        fs::remove_dir_all(package).unwrap();
    }

    #[test]
    fn install_package_to_device_rejects_existing_install_dir_without_modifying_it() {
        let package = temp_dir("device-package-existing-install");
        let device = temp_dir("device-root-existing-install");
        fs::write(package.join("zircon-package.toml"), "files_copied = 1").unwrap();
        let install_dir = device.join(package.file_name().unwrap());
        fs::create_dir_all(&install_dir).unwrap();
        fs::write(install_dir.join("keep.txt"), "keep").unwrap();

        let error =
            install_package_to_device(&DeviceInstallRequest::new(&package, &device)).unwrap_err();

        assert!(error
            .to_string()
            .contains("Device install already exists: "));
        assert_eq!(
            fs::read_to_string(install_dir.join("keep.txt")).unwrap(),
            "keep"
        );

        fs::remove_dir_all(package).unwrap();
        fs::remove_dir_all(device).unwrap();
    }

    fn temp_dir(label: &str) -> PathBuf {
        let root = std::env::temp_dir().join(format!(
            "zircon-hub-{label}-{}",
            crate::projects::now_unix_ms()
        ));
        fs::create_dir_all(&root).unwrap();
        root
    }
}
