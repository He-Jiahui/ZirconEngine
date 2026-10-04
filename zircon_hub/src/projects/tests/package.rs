use super::*;

#[test]
fn package_project_copies_project_files_and_writes_manifest() {
    let root = temp_dir("package-source");
    let output = temp_dir("package-output");
    fs::write(root.join("zircon-project.toml"), "name = \"Demo\"").unwrap();
    fs::create_dir_all(root.join("Assets")).unwrap();
    fs::write(root.join("Assets").join("mesh.txt"), "mesh").unwrap();
    fs::create_dir_all(root.join("target")).unwrap();
    fs::write(root.join("target").join("ignored.txt"), "ignored").unwrap();

    let request = ProjectPackageRequest {
        project_name: "Demo Project".to_string(),
        project_root: root.clone(),
        output_root: output.clone(),
        created_unix_ms: 42,
    };
    let TaskExecutionOutcome::Completed(report) =
        package_project(&request, &active_token()).unwrap()
    else {
        panic!("package fixture should complete");
    };

    assert!(report
        .package_dir
        .ends_with(Path::new("packages").join("demo-project-42")));
    assert!(report
        .package_dir
        .join(PACKAGE_PROJECT_DIR)
        .join("zircon-project.toml")
        .is_file());
    assert!(report
        .package_dir
        .join(PACKAGE_PROJECT_DIR)
        .join("Assets")
        .join("mesh.txt")
        .is_file());
    assert!(!report
        .package_dir
        .join(PACKAGE_PROJECT_DIR)
        .join("target")
        .exists());
    assert_eq!(report.files_copied, 2);
    assert!(fs::read_to_string(report.manifest_path)
        .unwrap()
        .contains("files_copied = 2"));

    fs::remove_dir_all(root).unwrap();
    fs::remove_dir_all(output).unwrap();
}

#[test]
fn package_project_rejects_output_inside_project() {
    let root = temp_dir("package-source-inside");
    let output = root.join("build-output");
    let request = ProjectPackageRequest::new("Demo", root.clone(), output);

    let error = package_project(&request, &active_token()).unwrap_err();
    fs::remove_dir_all(root).unwrap();

    assert!(error.to_string().contains("outside the project directory"));
}

#[test]
fn package_project_rejects_missing_output_inside_project_without_creating_directory() {
    let root = temp_dir("package-source-inside-missing");
    let output = root.join("build-output").join("nested");
    let request = ProjectPackageRequest::new("Demo", root.clone(), output.clone());

    let error = package_project(&request, &active_token()).unwrap_err();

    assert!(error.to_string().contains("outside the project directory"));
    assert!(
        !output.exists(),
        "rejected package output roots inside the project must not be created"
    );

    fs::remove_dir_all(root).unwrap();
}

#[test]
fn package_project_rejects_preexisting_unique_package_dir_without_deleting_it() {
    let root = temp_dir("package-source-existing-output");
    let output = temp_dir("package-existing-output");
    fs::write(root.join("zircon-project.toml"), "name = \"Demo\"").unwrap();
    let request = ProjectPackageRequest {
        project_name: "Demo Project".to_string(),
        project_root: root.clone(),
        output_root: output.clone(),
        created_unix_ms: 42,
    };
    let package_dir = output.join(PACKAGE_ROOT_DIR).join("demo-project-42");
    fs::create_dir_all(&package_dir).unwrap();
    fs::write(package_dir.join("keep.txt"), "keep").unwrap();

    let error = package_project(&request, &active_token()).unwrap_err();

    assert!(error
        .to_string()
        .contains("Package directory already exists: "));
    assert_eq!(
        fs::read_to_string(package_dir.join("keep.txt")).unwrap(),
        "keep"
    );

    fs::remove_dir_all(root).unwrap();
    fs::remove_dir_all(output).unwrap();
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
