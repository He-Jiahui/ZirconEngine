use std::fs;

use super::super::{NewProjectDraft, NewProjectTemplate, ProjectAuthority};
use super::temp_root;
use zircon_runtime::asset::project::{ProjectPaths, PROJECT_MANIFEST_FILE};

#[test]
fn existing_project_root_resolves_an_alias_to_the_canonical_identity() {
    let location = temp_root("existing-project-root-alias");
    let authority = ProjectAuthority::default();
    let created = authority
        .create_project(&NewProjectDraft {
            project_name: "Canonical Project".to_string(),
            location: location.to_string_lossy().into_owned(),
            template: NewProjectTemplate::RenderableEmpty,
        })
        .unwrap();
    let expected_root = created.root.clone();
    assert_eq!(created.identity().operation_path(), expected_root.as_path());
    assert_eq!(
        created.identity().display_path(),
        ProjectPaths::display_path(&expected_root)
    );
    drop(created);

    let alias = location.join("Project Alias");
    create_directory_link(&expected_root, &alias);

    let resolved = authority.resolve_existing_project_root(&alias).unwrap();
    assert_eq!(resolved, expected_root);
    let resolved_manifest = authority
        .resolve_existing_project_root(alias.join("zircon-project.toml"))
        .unwrap();
    assert_eq!(resolved_manifest, expected_root);

    let opened = authority.open_project(&alias).unwrap();
    assert_eq!(opened.project().paths().root(), expected_root.as_path());
    assert_eq!(opened.identity().operation_path(), expected_root.as_path());
    assert_eq!(
        opened.identity().display_path(),
        ProjectPaths::display_path(&expected_root)
    );
    drop(opened);

    let resolved = ProjectPaths::resolve_existing(&alias).unwrap();
    let opened = authority.open_resolved_project(&resolved).unwrap();
    assert_eq!(opened.project().paths().root(), expected_root.as_path());
    drop(opened);
    fs::remove_dir_all(location).unwrap();
}

#[test]
fn project_authority_resolves_a_manifest_alias_once_before_opening() {
    let location = temp_root("manifest-alias-single-resolution");
    let authority = ProjectAuthority::default();
    let created = authority
        .create_project(&NewProjectDraft {
            project_name: "Manifest Identity Project".to_string(),
            location: location.to_string_lossy().into_owned(),
            template: NewProjectTemplate::RenderableEmpty,
        })
        .unwrap();
    let expected_root = created.root.clone();
    drop(created);

    let alias = location.join("Manifest Project Alias");
    create_directory_link(&expected_root, &alias);
    let manifest_alias = alias.join("zircon-project.toml");

    let resolved = authority
        .resolve_existing_project_root_with_identity(&manifest_alias)
        .unwrap();
    assert_eq!(resolved.operation_path(), expected_root.as_path());

    let opened = authority.open_project(&manifest_alias).unwrap();
    assert_eq!(opened.project().paths().root(), expected_root.as_path());
    drop(opened);
    fs::remove_dir_all(location).unwrap();
}

#[test]
fn project_authority_keeps_a_manifest_named_directory_as_the_project_root() {
    let location = temp_root("manifest-named-project-root");
    let project_root = location.join(PROJECT_MANIFEST_FILE);
    fs::create_dir_all(&project_root).unwrap();
    fs::write(project_root.join(PROJECT_MANIFEST_FILE), "[project]\n").unwrap();

    let resolved = ProjectAuthority::default()
        .resolve_existing_project_root_with_identity(&project_root)
        .unwrap();

    assert_eq!(
        resolved,
        ProjectPaths::resolve_existing(&project_root).unwrap()
    );
    fs::remove_dir_all(location).unwrap();
}

#[test]
fn create_project_resolves_an_alias_location_to_the_physical_target() {
    let parent = temp_root("create-project-alias-location");
    let physical_location = parent.join("physical-location");
    fs::create_dir(&physical_location).unwrap();
    let alias_location = parent.join("aliased-location");
    create_directory_link(&physical_location, &alias_location);

    let created = ProjectAuthority::default()
        .create_project(&NewProjectDraft {
            project_name: "Alias Created Project".to_string(),
            location: alias_location.to_string_lossy().into_owned(),
            template: NewProjectTemplate::RenderableEmpty,
        })
        .unwrap();
    let expected_root =
        ProjectPaths::resolve_existing_path(physical_location.join("Alias Created Project"))
            .unwrap();
    assert_eq!(created.root, expected_root);
    assert_eq!(created.project().paths().root(), expected_root.as_path());

    drop(created);
    fs::remove_dir_all(parent).unwrap();
}

#[cfg(unix)]
fn create_directory_link(target: &std::path::Path, link: &std::path::Path) {
    std::os::unix::fs::symlink(target, link).expect("create project-root alias fixture");
}

#[cfg(windows)]
fn create_directory_link(target: &std::path::Path, link: &std::path::Path) {
    let command = format!(r#"mklink /J "{}" "{}""#, link.display(), target.display());
    let output = std::process::Command::new("cmd")
        .args(["/D", "/S", "/C"])
        .arg(command)
        .output()
        .expect("start mklink for project-root junction fixture");
    assert!(
        output.status.success(),
        "create project-root junction fixture failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
}

#[cfg(windows)]
#[test]
fn project_authority_resolves_directory_symbolic_link_roots() {
    let location = temp_root("existing-project-root-symbolic-link");
    let authority = ProjectAuthority::default();
    let created = authority
        .create_project(&NewProjectDraft {
            project_name: "Symbolic Link Project".to_string(),
            location: location.to_string_lossy().into_owned(),
            template: NewProjectTemplate::RenderableEmpty,
        })
        .unwrap();
    let expected_root = created.root.clone();
    drop(created);

    let alias = location.join("Project Symbolic Link");
    std::os::windows::fs::symlink_dir(&expected_root, &alias)
        .expect("create project-root symbolic-link fixture");

    assert_eq!(
        authority.resolve_existing_project_root(&alias).unwrap(),
        expected_root
    );
    assert_eq!(
        authority
            .resolve_existing_project_root(alias.join("zircon-project.toml"))
            .unwrap(),
        expected_root
    );

    fs::remove_dir_all(location).unwrap();
}

#[cfg(windows)]
#[test]
fn project_authority_resolves_subst_roots_and_uncreated_tails() {
    let parent = temp_root("project-root-subst");
    let physical_location = parent.join("physical-location");
    fs::create_dir(&physical_location).unwrap();
    let authority = ProjectAuthority::default();
    let created = authority
        .create_project(&NewProjectDraft {
            project_name: "Existing Project".to_string(),
            location: physical_location.to_string_lossy().into_owned(),
            template: NewProjectTemplate::RenderableEmpty,
        })
        .unwrap();
    let expected_root = created.root.clone();
    drop(created);

    let mut subst = SubstDrive::mount(&physical_location);
    let existing_alias = subst.path().join("Existing Project");
    assert_eq!(
        authority
            .resolve_existing_project_root(&existing_alias)
            .unwrap(),
        expected_root
    );
    assert_eq!(
        authority
            .resolve_existing_project_root(existing_alias.join("zircon-project.toml"))
            .unwrap(),
        expected_root
    );

    let created = authority
        .create_project(&NewProjectDraft {
            project_name: "Created Through Subst".to_string(),
            location: subst.path().to_string_lossy().into_owned(),
            template: NewProjectTemplate::RenderableEmpty,
        })
        .unwrap();
    assert_eq!(
        created.root,
        ProjectPaths::resolve_existing_path(physical_location.join("Created Through Subst"))
            .unwrap()
    );

    drop(created);
    subst.unmount();
    fs::remove_dir_all(parent).unwrap();
}

#[cfg(windows)]
struct SubstDrive {
    drive: String,
    root: std::path::PathBuf,
    mounted: bool,
}

#[cfg(windows)]
impl SubstDrive {
    fn mount(target: &std::path::Path) -> Self {
        for letter in b'D'..=b'Z' {
            let drive = format!("{}:", char::from(letter));
            let root = std::path::PathBuf::from(format!("{drive}\\"));
            if root.exists() {
                continue;
            }
            let output = std::process::Command::new("subst")
                .arg(&drive)
                .arg(target)
                .output()
                .expect("start subst for project-root fixture");
            if output.status.success() {
                return Self {
                    drive,
                    root,
                    mounted: true,
                };
            }
        }
        panic!("reserve a free SUBST drive for project-root fixture");
    }

    fn path(&self) -> &std::path::Path {
        &self.root
    }

    fn unmount(&mut self) {
        let output = std::process::Command::new("subst")
            .arg(&self.drive)
            .arg("/D")
            .output()
            .expect("start SUBST fixture cleanup");
        assert!(
            output.status.success(),
            "remove SUBST fixture {} failed: {}",
            self.drive,
            String::from_utf8_lossy(&output.stderr)
        );
        self.mounted = false;
    }
}

#[cfg(windows)]
impl Drop for SubstDrive {
    fn drop(&mut self) {
        if self.mounted {
            match std::process::Command::new("subst")
                .arg(&self.drive)
                .arg("/D")
                .output()
            {
                Ok(output) if output.status.success() => {}
                Ok(output) => eprintln!(
                    "remove SUBST fixture {} failed: {}",
                    self.drive,
                    String::from_utf8_lossy(&output.stderr)
                ),
                Err(error) => eprintln!("remove SUBST fixture {} failed: {error}", self.drive),
            }
        }
    }
}
