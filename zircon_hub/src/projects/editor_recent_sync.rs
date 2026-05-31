use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::error::HubError;

use super::metadata::{project_metadata_key, project_paths_match};
use super::recent_project::{RecentProject, RECENT_PROJECT_LIMIT};

const EDITOR_STARTUP_SESSION_KEY: &str = "editor.startup.session";

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
struct EditorStartupSession {
    pub last_project_path: Option<String>,
    #[serde(default)]
    pub recent_projects: Vec<EditorRecentProjectEntry>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
struct EditorRecentProjectEntry {
    pub display_name: String,
    pub path: String,
    pub last_opened_unix_ms: u64,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct EditorRecentProjectSession {
    pub last_project_path: Option<PathBuf>,
    pub recent_projects: Vec<RecentProject>,
}

pub fn load_editor_recent_projects(path: impl AsRef<Path>) -> Result<Vec<RecentProject>, HubError> {
    Ok(load_editor_recent_project_session(path)?.recent_projects)
}

pub fn load_editor_recent_project_session(
    path: impl AsRef<Path>,
) -> Result<EditorRecentProjectSession, HubError> {
    let path = path.as_ref();
    if !path.exists() {
        return Ok(EditorRecentProjectSession::default());
    }

    let text = fs::read_to_string(path)?;
    let values = serde_json::from_str::<HashMap<String, Value>>(&text)?;
    let Some(value) = values.get(EDITOR_STARTUP_SESSION_KEY) else {
        return Ok(EditorRecentProjectSession::default());
    };
    let session = serde_json::from_value::<EditorStartupSession>(value.clone())?;
    Ok(EditorRecentProjectSession {
        last_project_path: session.last_project_path.map(PathBuf::from),
        recent_projects: session
            .recent_projects
            .into_iter()
            .map(|entry| RecentProject {
                display_name: entry.display_name,
                path: PathBuf::from(entry.path),
                last_opened_unix_ms: entry.last_opened_unix_ms,
            })
            .collect(),
    })
}

pub fn save_editor_recent_projects(
    path: impl AsRef<Path>,
    recent_projects: &[RecentProject],
) -> Result<(), HubError> {
    save_editor_recent_projects_with_last_project(path, recent_projects, None)
}

pub fn save_editor_recent_projects_with_last_project(
    path: impl AsRef<Path>,
    recent_projects: &[RecentProject],
    last_project_path: Option<&Path>,
) -> Result<(), HubError> {
    let path = path.as_ref();
    let mut values = if path.exists() {
        serde_json::from_str::<HashMap<String, Value>>(&fs::read_to_string(path)?)?
    } else {
        HashMap::new()
    };
    let recent_projects = merge_recent_projects(recent_projects.iter().cloned(), Vec::new());
    let existing_session = values
        .get(EDITOR_STARTUP_SESSION_KEY)
        .cloned()
        .map(serde_json::from_value::<EditorStartupSession>)
        .transpose()?;
    let last_project_path = last_project_path
        .map(|path| path.to_string_lossy().into_owned())
        .or_else(|| existing_session.and_then(|session| session.last_project_path))
        .filter(|path| {
            recent_projects
                .iter()
                .any(|project| project_paths_match(&project.path, Path::new(path)))
        });
    let session = EditorStartupSession {
        last_project_path,
        recent_projects: recent_projects
            .into_iter()
            .map(|entry| EditorRecentProjectEntry {
                display_name: entry.display_name,
                path: entry.path.to_string_lossy().into_owned(),
                last_opened_unix_ms: entry.last_opened_unix_ms,
            })
            .collect(),
    };
    values.insert(
        EDITOR_STARTUP_SESSION_KEY.to_string(),
        serde_json::to_value(session)?,
    );
    if let Some(parent) = path.parent() {
        if !parent.as_os_str().is_empty() {
            fs::create_dir_all(parent)?;
        }
    }
    fs::write(path, serde_json::to_string_pretty(&values)?)?;
    Ok(())
}

pub fn merge_recent_projects<I, J>(hub: I, editor: J) -> Vec<RecentProject>
where
    I: IntoIterator<Item = RecentProject>,
    J: IntoIterator<Item = RecentProject>,
{
    let mut by_path: HashMap<String, RecentProject> = HashMap::new();
    for entry in hub.into_iter().chain(editor) {
        let key = project_metadata_key(&entry.path);
        match by_path.get(&key) {
            Some(existing) if existing.last_opened_unix_ms > entry.last_opened_unix_ms => {}
            Some(existing)
                if existing.last_opened_unix_ms == entry.last_opened_unix_ms
                    && existing.display_name <= entry.display_name => {}
            _ => {
                by_path.insert(key, entry);
            }
        }
    }
    let mut merged = by_path.into_values().collect::<Vec<_>>();
    merged.sort_by(|left, right| {
        right
            .last_opened_unix_ms
            .cmp(&left.last_opened_unix_ms)
            .then_with(|| left.path.cmp(&right.path))
    });
    merged.truncate(RECENT_PROJECT_LIMIT);
    merged
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn recent_merge_deduplicates_by_path_and_keeps_newest_eight() {
        let hub = vec![
            RecentProject::new("Old", "E:/Projects/Game", 1),
            RecentProject::new("Other", "E:/Projects/Other", 2),
        ];
        let mut editor = (0..10)
            .map(|index| {
                RecentProject::new(
                    format!("Project{index}"),
                    format!("E:/Projects/Project{index}"),
                    10 + index,
                )
            })
            .collect::<Vec<_>>();
        editor.push(RecentProject::new("New", "E:/Projects/Game", 99));

        let merged = merge_recent_projects(hub, editor);

        assert_eq!(merged.len(), RECENT_PROJECT_LIMIT);
        assert_eq!(merged[0].display_name, "New");
        assert_eq!(
            merged
                .iter()
                .filter(|entry| entry.path == PathBuf::from("E:/Projects/Game"))
                .count(),
            1
        );
    }

    #[test]
    fn editor_recent_writer_preserves_unrelated_config_keys() {
        let root =
            std::env::temp_dir().join(format!("zircon_hub_recent_writer_{}", std::process::id()));
        let _ = fs::remove_dir_all(&root);
        fs::create_dir_all(&root).unwrap();
        let path = root.join("config.json");
        fs::write(&path, r#"{"other.key":true}"#).unwrap();

        save_editor_recent_projects(&path, &[RecentProject::new("Game", "E:/Projects/Game", 42)])
            .unwrap();

        let values =
            serde_json::from_str::<HashMap<String, Value>>(&fs::read_to_string(&path).unwrap())
                .unwrap();
        assert_eq!(values["other.key"], Value::Bool(true));
        assert_eq!(
            values[EDITOR_STARTUP_SESSION_KEY]["recent_projects"][0]["path"],
            Value::String("E:/Projects/Game".to_string())
        );
        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn editor_recent_loader_returns_recent_projects_and_last_project() {
        let root = std::env::temp_dir().join(format!(
            "zircon_hub_recent_loader_last_project_{}",
            std::process::id()
        ));
        let _ = fs::remove_dir_all(&root);
        fs::create_dir_all(&root).unwrap();
        let path = root.join("config.json");
        fs::write(
            &path,
            r#"{"editor.startup.session":{"last_project_path":"E:/Projects/Game","recent_projects":[{"display_name":"Game","path":"E:/Projects/Game","last_opened_unix_ms":42}]}}"#,
        )
        .unwrap();

        let session = load_editor_recent_project_session(&path).unwrap();

        assert_eq!(
            session.last_project_path,
            Some(PathBuf::from("E:/Projects/Game"))
        );
        assert_eq!(session.recent_projects.len(), 1);
        assert_eq!(session.recent_projects[0].display_name, "Game");
        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn editor_recent_writer_does_not_emit_hub_project_metadata() {
        let root = std::env::temp_dir().join(format!(
            "zircon_hub_recent_metadata_boundary_{}",
            std::process::id()
        ));
        let _ = fs::remove_dir_all(&root);
        fs::create_dir_all(&root).unwrap();
        let path = root.join("config.json");

        save_editor_recent_projects(&path, &[RecentProject::new("Game", "E:/Projects/Game", 42)])
            .unwrap();

        let text = fs::read_to_string(&path).unwrap();
        assert!(!text.contains("pinned"));
        assert!(!text.contains("engine_id"));
        assert!(!text.contains("last_selected_template"));
        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn editor_recent_writer_preserves_existing_last_project_without_override_when_recent() {
        let root = std::env::temp_dir().join(format!(
            "zircon_hub_recent_last_project_{}",
            std::process::id()
        ));
        let _ = fs::remove_dir_all(&root);
        fs::create_dir_all(&root).unwrap();
        let path = root.join("config.json");
        fs::write(
            &path,
            r#"{"editor.startup.session":{"last_project_path":"E:/Projects/Last","recent_projects":[]}}"#,
        )
        .unwrap();

        save_editor_recent_projects(
            &path,
            &[
                RecentProject::new("Last", "E:/Projects/Last", 99),
                RecentProject::new("Game", "E:/Projects/Game", 42),
            ],
        )
        .unwrap();

        let values =
            serde_json::from_str::<HashMap<String, Value>>(&fs::read_to_string(&path).unwrap())
                .unwrap();
        assert_eq!(
            values[EDITOR_STARTUP_SESSION_KEY]["last_project_path"],
            Value::String("E:/Projects/Last".to_string())
        );
        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn editor_recent_writer_clears_last_project_when_it_is_no_longer_recent() {
        let root = std::env::temp_dir().join(format!(
            "zircon_hub_recent_last_project_clear_{}",
            std::process::id()
        ));
        let _ = fs::remove_dir_all(&root);
        fs::create_dir_all(&root).unwrap();
        let path = root.join("config.json");
        fs::write(
            &path,
            r#"{"editor.startup.session":{"last_project_path":"E:/Projects/Removed","recent_projects":[]}}"#,
        )
        .unwrap();

        save_editor_recent_projects(&path, &[RecentProject::new("Game", "E:/Projects/Game", 42)])
            .unwrap();

        let values =
            serde_json::from_str::<HashMap<String, Value>>(&fs::read_to_string(&path).unwrap())
                .unwrap();
        assert!(values[EDITOR_STARTUP_SESSION_KEY]["last_project_path"].is_null());
        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn editor_recent_writer_can_override_last_project_after_launch() {
        let root = std::env::temp_dir().join(format!(
            "zircon_hub_recent_last_project_override_{}",
            std::process::id()
        ));
        let _ = fs::remove_dir_all(&root);
        fs::create_dir_all(&root).unwrap();
        let path = root.join("config.json");
        fs::write(
            &path,
            r#"{"editor.startup.session":{"last_project_path":"E:/Projects/Last","recent_projects":[]}}"#,
        )
        .unwrap();

        save_editor_recent_projects_with_last_project(
            &path,
            &[RecentProject::new("Game", "E:/Projects/Game", 42)],
            Some(Path::new("E:/Projects/Game")),
        )
        .unwrap();

        let values =
            serde_json::from_str::<HashMap<String, Value>>(&fs::read_to_string(&path).unwrap())
                .unwrap();
        assert_eq!(
            values[EDITOR_STARTUP_SESSION_KEY]["last_project_path"],
            Value::String("E:/Projects/Game".to_string())
        );
        let _ = fs::remove_dir_all(root);
    }
}
