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

    let TaskExecutionOutcome::Completed(report) = install_package_to_device(
        &DeviceInstallRequest::new(package.clone(), device.clone()),
        &active_token(),
    )
    .unwrap() else {
        panic!("install fixture should complete");
    };

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
    let error = install_package_to_device(
        &DeviceInstallRequest::new(&package, &device),
        &active_token(),
    )
    .unwrap_err();
    fs::remove_dir_all(package).unwrap();

    assert!(error.to_string().contains("outside the package directory"));
}

#[test]
fn install_package_to_device_rejects_missing_device_root_inside_package_without_creating_directory()
{
    let package = temp_dir("device-package-inside-missing");
    let device = package.join("device").join("nested");

    let error = install_package_to_device(
        &DeviceInstallRequest::new(&package, &device),
        &active_token(),
    )
    .unwrap_err();

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

    let error = install_package_to_device(
        &DeviceInstallRequest::new(&package, &device),
        &active_token(),
    )
    .unwrap_err();

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

fn active_token() -> TaskCancellationToken {
    TaskCancellationToken::new(1)
}
