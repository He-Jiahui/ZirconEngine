use std::{fs, path::Path, process::Command, process::Stdio};

use crate::engines::{source_engine_id, SourceEngineInstall};
use crate::projects::RecentProject;
use crate::settings::HubConfig;

use super::super::HubRuntimeSession;
use super::*;

#[test]
fn team_overview_prefers_selected_project_git_repository_over_source_engine() {
    let Some(git) = git_command() else {
        return;
    };
    let temp = temp_test_dir("zircon-hub-team-project-first");
    let project_repo = create_git_repo(
        &git,
        &temp.join("Game"),
        "Project Dev",
        "project@example.com",
    );
    let source_repo = create_git_repo(
        &git,
        &temp.join("ZirconEngine"),
        "Engine Dev",
        "engine@example.com",
    );
    let mut session = session_with_source(&temp, &source_repo);
    session.config.recent_projects = vec![RecentProject::fixture("Game", &project_repo, 10)];
    session.selected_project_path = Some(project_repo.clone());

    session
        .refresh_selected_project_scoped_views()
        .expect("project-scoped views should refresh");

    assert_eq!(session.team_overview.repository_path, project_repo);
    assert_eq!(session.team_overview.identity_name, "Project Dev");
    assert_eq!(session.team_overview.identity_email, "project@example.com");
    assert_eq!(session.team_overview.members[0].name, "Project Dev");

    fs::remove_dir_all(temp).unwrap();
}

#[test]
fn team_overview_falls_back_to_source_engine_git_repository_without_selected_project() {
    let Some(git) = git_command() else {
        return;
    };
    let temp = temp_test_dir("zircon-hub-team-source-fallback");
    let source_repo = create_git_repo(
        &git,
        &temp.join("ZirconEngine"),
        "Engine Dev",
        "engine@example.com",
    );
    let mut session = session_with_source(&temp, &source_repo);
    session.selected_project_path = None;
    session.config.recent_projects.clear();

    session
        .refresh_source_scoped_views()
        .expect("source-scoped views should refresh");

    assert_eq!(session.team_overview.repository_path, source_repo);
    assert_eq!(session.team_overview.identity_name, "Engine Dev");
    assert_eq!(session.team_overview.identity_email, "engine@example.com");
    assert_eq!(session.team_overview.members[0].email, "engine@example.com");

    fs::remove_dir_all(temp).unwrap();
}

fn session_with_source(temp: &Path, source: &Path) -> HubRuntimeSession {
    let config_path = temp.join("hub.toml");
    let shared_recent_projects_path = temp.join("recent_projects.json");
    let mut config = HubConfig::default();
    config.settings.default_project_dir = temp.join("projects");
    config.settings.default_source_dir = source.to_path_buf();
    config.settings.default_build_output_dir = temp.join("out");
    config.engines.push(SourceEngineInstall {
        id: source_engine_id(source),
        display_name: "Local Source".to_string(),
        source_dir: source.to_path_buf(),
        output_dir: temp.join("out"),
        last_build_unix_ms: None,
        build_history: Vec::new(),
    });
    config.active_engine_id = Some(source_engine_id(source));
    config.runtime.new_project_engine_id = Some(source_engine_id(source));
    config.save(&config_path).unwrap();
    fs::write(
        &shared_recent_projects_path,
        r#"{"protocol_version":1,"projects":[]}"#,
    )
    .unwrap();
    HubRuntimeSession::load_from_paths(config_path, shared_recent_projects_path).unwrap()
}

fn create_git_repo(git: &str, root: &Path, name: &str, email: &str) -> PathBuf {
    fs::create_dir_all(root).unwrap();
    run_git(git, root, &["init"]);
    run_git(git, root, &["config", "user.name", name]);
    run_git(git, root, &["config", "user.email", email]);
    run_git(git, root, &["config", "commit.gpgSign", "false"]);
    fs::write(root.join("README.md"), format!("# {name}\n")).unwrap();
    fs::write(
        root.join("zircon-project.toml"),
        format!("name = \"{name}\"\n"),
    )
    .unwrap();
    run_git(git, root, &["add", "."]);
    run_git(git, root, &["commit", "-m", "initial"]);
    root.to_path_buf()
}

fn run_git(git: &str, root: &Path, args: &[&str]) {
    let output = Command::new(git)
        .arg("-C")
        .arg(root)
        .args(args)
        .stdout(Stdio::null())
        .stderr(Stdio::piped())
        .output()
        .expect("git command should run");
    assert!(
        output.status.success(),
        "git {:?} failed: {}",
        args,
        String::from_utf8_lossy(&output.stderr)
    );
}

fn git_command() -> Option<String> {
    Command::new("git")
        .arg("--version")
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .ok()
        .filter(|status| status.success())
        .map(|_| "git".to_string())
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
