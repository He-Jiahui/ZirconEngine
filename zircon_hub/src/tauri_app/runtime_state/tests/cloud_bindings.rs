use std::{fs, path::PathBuf};

use crate::projects::RecentProject;
use crate::settings::HubConfig;

use super::*;

fn fixture() -> (PathBuf, HubRuntimeSession, CloudAccountScope) {
    let root =
        PathBuf::from(std::env::var_os("CARGO_TARGET_DIR").expect("managed target")).join(format!(
            "hub-cloud-binding-{}-{}",
            std::process::id(),
            crate::projects::now_unix_ms()
        ));
    let project = root.join("Local");
    fs::create_dir_all(&project).unwrap();
    let guid = ProjectGuid::new();
    fs::write(project.join("zircon-project.toml"), format!(
        "name = 'Local'\ndefault_scene = 'res://scenes/main.scene.toml'\nformat_version = {}\nlibrary_version = 1\nproject_guid = '{guid}'\nasset_roots = ['res']\n",
        zircon_runtime_interface::project::PROJECT_MANIFEST_FORMAT_VERSION
    )).unwrap();
    let mut config = HubConfig::default();
    config
        .recent_projects
        .push(RecentProject::from_project_path(&project, 1).unwrap());
    config.runtime.selected_project_path = Some(project);
    let config_path = root.join("hub.toml");
    config.save(&config_path).unwrap();
    let session =
        HubRuntimeSession::load_from_paths(config_path, root.join("recent.json")).unwrap();
    let account = CloudAccountScope {
        environment: crate::projects::CloudBindingEnvironment {
            issuer: "https://identity.example".into(),
            client_id: "hub".into(),
            service_url: "https://service.example".into(),
        },
        subject: "user-1".into(),
    };
    (root, session, account)
}

#[test]
fn selected_binding_survives_reopen_but_never_follows_other_account_path_or_manifest() {
    let (root, mut session, account) = fixture();
    let local = session.selected_local_cloud_project().unwrap();
    let binding = session
        .attach_cloud_binding(
            account.clone(),
            &local,
            "00000000-0000-4000-8000-000000000001".into(),
            "00000000-0000-4000-8000-000000000002".into(),
        )
        .unwrap();
    assert_eq!(
        session.selected_cloud_binding(&account),
        Some(binding.clone())
    );
    let reopened =
        HubRuntimeSession::load_from_paths(root.join("hub.toml"), root.join("recent.json"))
            .unwrap();
    assert_eq!(reopened.selected_cloud_binding(&account), Some(binding));
    let mut wrong_account = account.clone();
    wrong_account.subject = "other-user".into();
    assert!(reopened.selected_cloud_binding(&wrong_account).is_none());
    let mut wrong_service = account.clone();
    wrong_service.environment.service_url = "https://other.example".into();
    assert!(reopened.selected_cloud_binding(&wrong_service).is_none());
    session.selected_project_path = None;
    assert!(session.selected_cloud_binding(&account).is_none());
    let copied = root.join("Copied");
    fs::create_dir_all(&copied).unwrap();
    fs::copy(
        reopened
            .selected_project_path
            .as_ref()
            .unwrap()
            .join("zircon-project.toml"),
        copied.join("zircon-project.toml"),
    )
    .unwrap();
    session.config.recent_projects[0].path = copied.clone();
    session.selected_project_path = Some(copied);
    assert_eq!(
        session.selected_local_cloud_project().unwrap().guid,
        local.guid
    );
    assert!(session.selected_cloud_binding(&account).is_none());
    assert!(session
        .attach_cloud_binding(
            account.clone(),
            &local,
            "00000000-0000-4000-8000-000000000001".into(),
            "00000000-0000-4000-8000-000000000002".into(),
        )
        .is_err());
    session.config.recent_projects[0].path = reopened.selected_project_path.clone().unwrap();
    session.selected_project_path = reopened.selected_project_path.clone();
    let manifest = session
        .selected_project_path
        .as_ref()
        .unwrap()
        .join("zircon-project.toml");
    let text = fs::read_to_string(&manifest).unwrap();
    fs::write(
        &manifest,
        text.replace(&local.guid.to_string(), &ProjectGuid::new().to_string()),
    )
    .unwrap();
    assert!(session.selected_cloud_binding(&account).is_none());
    fs::remove_dir_all(root).unwrap();
}
