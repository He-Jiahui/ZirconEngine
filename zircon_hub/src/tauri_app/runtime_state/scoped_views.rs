use std::path::PathBuf;

use crate::assets::discover_asset_catalog_for_scope;
use crate::error::HubError;
use crate::learn::discover_learn_catalog_for_scope;
use crate::plugins::discover_plugin_catalog_with_project_roots;
use crate::projects::project_filesystem_path_key;
use crate::team::discover_team_overview;

use super::HubRuntimeSession;

impl HubRuntimeSession {
    pub(super) fn refresh_source_scoped_views(&mut self) -> Result<(), HubError> {
        self.refresh_asset_catalog()?;
        self.refresh_learn_catalog()?;
        self.refresh_plugin_catalog()?;
        self.refresh_team_overview()
    }

    pub(super) fn refresh_selected_project_scoped_views(&mut self) -> Result<(), HubError> {
        self.refresh_asset_catalog()?;
        self.refresh_learn_catalog()?;
        self.refresh_plugin_catalog()?;
        self.refresh_team_overview()
    }

    fn refresh_asset_catalog(&mut self) -> Result<(), HubError> {
        self.asset_catalog = discover_asset_catalog_for_scope(
            self.selected_project_catalog_root(),
            self.config
                .recent_projects
                .iter()
                .map(|project| project.path.clone())
                .collect::<Vec<_>>(),
            self.source_engine_catalog_roots(),
        )?;
        Ok(())
    }

    fn refresh_learn_catalog(&mut self) -> Result<(), HubError> {
        self.learn_catalog = discover_learn_catalog_for_scope(
            self.selected_project_catalog_root(),
            self.source_engine_catalog_roots(),
        )?;
        Ok(())
    }

    fn refresh_plugin_catalog(&mut self) -> Result<(), HubError> {
        self.plugin_catalog = discover_plugin_catalog_with_project_roots(
            self.selected_project_catalog_root().into_iter(),
            self.source_engine_catalog_roots(),
        )?;
        Ok(())
    }

    fn refresh_team_overview(&mut self) -> Result<(), HubError> {
        let mut roots = Vec::new();
        if let Some(project_root) = self.selected_project_catalog_root() {
            push_unique_root(&mut roots, project_root);
        }
        for source_root in self.source_engine_catalog_roots() {
            push_unique_root(&mut roots, source_root);
        }
        self.team_overview = discover_team_overview(roots)?;
        Ok(())
    }

    fn selected_project_catalog_root(&self) -> Option<PathBuf> {
        self.snapshot()
            .scope()
            .selected_project()
            .map(|project| project.path.clone())
    }

    fn source_engine_catalog_roots(&self) -> Vec<PathBuf> {
        let mut roots = Vec::new();
        let scope = self.snapshot().scope();
        let Some(engine_id) = scope.source_engine.engine_id() else {
            return roots;
        };
        if let Some(engine) = self
            .config
            .engines
            .iter()
            .find(|engine| engine.id == engine_id)
        {
            push_development_roots(&mut roots, engine.source_dir.clone());
        }
        roots
    }
}

fn push_unique_root(roots: &mut Vec<PathBuf>, path: PathBuf) {
    if path.as_os_str().is_empty() {
        return;
    }
    let candidate_key = project_filesystem_path_key(&path);
    if roots
        .iter()
        .any(|root| project_filesystem_path_key(root) == candidate_key)
    {
        return;
    }
    roots.push(path);
}

fn push_development_roots(roots: &mut Vec<PathBuf>, source_dir: PathBuf) {
    push_unique_root(roots, source_dir);
    if let Ok(current_dir) = std::env::current_dir() {
        push_unique_root(roots, current_dir);
    }
    if let Some(compiled_repo_root) = compiled_repo_root() {
        push_unique_root(roots, compiled_repo_root);
    }
}

fn compiled_repo_root() -> Option<PathBuf> {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .map(|path| path.to_path_buf())
}

#[cfg(test)]
mod tests {
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
        session.config.recent_projects = vec![RecentProject::new("Game", &project_repo, 10)];
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
        let editor_config_path = temp.join("editor.json");
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
            &editor_config_path,
            r#"{"editor.startup.session":{"recent_projects":[]}}"#,
        )
        .unwrap();
        HubRuntimeSession::load_from_paths(config_path, editor_config_path).unwrap()
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
}
