use std::fs;
use std::path::{Path, PathBuf};

use serde_json::Value;
use zircon_asset::{ProjectManifest, ProjectPaths};

use crate::default_constraints_for_content;
use crate::layout::{LayoutCommand, MainPageId};
use crate::project::{project_root_path, EditorProjectDocument};
use crate::view::{
    PreferredHost, ViewDescriptor, ViewDescriptorId, ViewHost, ViewInstance, ViewInstanceId,
    ViewKind,
};
use crate::workbench::startup::{
    now_unix_ms, EditorSessionMode, EditorStartupSessionDocument, NewProjectDraft,
    RecentProjectEntry, RecentProjectValidation, StoredStartupSession, STARTUP_SESSION_KEY,
    WELCOME_DESCRIPTOR_ID, WELCOME_INSTANCE_ID, WELCOME_PAGE_ID,
};
use crate::ViewContentKind;

use super::editor_error::EditorError;
use super::editor_manager::EditorManager;

impl EditorManager {
    pub fn resolve_startup_session(&self) -> Result<EditorStartupSessionDocument, EditorError> {
        let stored = self.load_startup_session()?;
        let recent_projects =
            stored.recent_projects_with_validation(|path| self.validate_recent_project(path));
        let mut session = EditorStartupSessionDocument {
            recent_projects,
            draft: NewProjectDraft::renderable_empty_default(),
            ..EditorStartupSessionDocument::default()
        };

        if let Some(path) = stored.last_project_path.as_deref() {
            if self.validate_recent_project(path) == RecentProjectValidation::Valid {
                let document = self.open_project(path)?;
                self.dismiss_welcome_page()?;
                session.mode = EditorSessionMode::Project;
                session.project = Some(document);
                session.status_message = format!("Reopened {}", path);
                return Ok(session);
            }

            self.show_welcome_page()?;
            session.mode = EditorSessionMode::Welcome;
            session.status_message = format!(
                "Last project is unavailable: {path}. Choose a recent project or create a new one."
            );
            return Ok(session);
        }

        self.show_welcome_page()?;
        Ok(session)
    }

    pub fn open_project_and_remember(
        &self,
        path: impl AsRef<Path>,
    ) -> Result<EditorStartupSessionDocument, EditorError> {
        let document = self.open_project(&path)?;
        self.update_recent_project(&document.root_path, document.manifest.name.as_str())?;
        self.dismiss_welcome_page()?;

        Ok(EditorStartupSessionDocument {
            mode: EditorSessionMode::Project,
            project: Some(document),
            recent_projects: self.recent_projects_snapshot()?,
            draft: NewProjectDraft::renderable_empty_default(),
            status_message: "Project opened".to_string(),
        })
    }

    pub fn create_project_and_open(
        &self,
        draft: NewProjectDraft,
    ) -> Result<EditorStartupSessionDocument, EditorError> {
        let root = EditorProjectDocument::create_renderable_template(&draft)
            .map_err(|error| EditorError::Project(error.to_string()))?;
        self.open_project_and_remember(root)
    }

    pub fn recent_projects_snapshot(&self) -> Result<Vec<RecentProjectEntry>, EditorError> {
        Ok(self
            .load_startup_session()?
            .recent_projects_with_validation(|path| self.validate_recent_project(path)))
    }

    pub fn forget_recent_project(&self, path: impl AsRef<Path>) -> Result<(), EditorError> {
        let mut stored = self.load_startup_session()?;
        stored.forget_recent_project(path.as_ref().to_string_lossy().as_ref());
        self.save_startup_session(&stored)
    }

    pub fn update_recent_project(
        &self,
        path: impl AsRef<Path>,
        display_name: &str,
    ) -> Result<(), EditorError> {
        let path = canonical_project_root(path.as_ref())
            .map_err(|error| EditorError::Project(error.to_string()))?;
        let mut stored = self.load_startup_session()?;
        stored.update_recent_project(path.to_string_lossy().as_ref(), display_name, now_unix_ms());
        self.save_startup_session(&stored)
    }

    fn load_startup_session(&self) -> Result<StoredStartupSession, EditorError> {
        let Some(value) = self.config_manager()?.get_value(STARTUP_SESSION_KEY) else {
            return Ok(StoredStartupSession::default());
        };
        serde_json::from_value(value).map_err(|error| EditorError::Project(error.to_string()))
    }

    fn save_startup_session(&self, session: &StoredStartupSession) -> Result<(), EditorError> {
        self.config_manager()?
            .set_value(
                STARTUP_SESSION_KEY,
                serde_json::to_value(session)
                    .map_err(|error| EditorError::Project(error.to_string()))?,
            )
            .map_err(|error| EditorError::Project(error.to_string()))
    }

    fn validate_recent_project(&self, path: &str) -> RecentProjectValidation {
        let Ok(root) = canonical_project_root(Path::new(path)) else {
            return RecentProjectValidation::Missing;
        };
        if !root.exists() {
            return RecentProjectValidation::Missing;
        }
        let Ok(paths) = ProjectPaths::from_root(&root) else {
            return RecentProjectValidation::InvalidProject;
        };
        if !paths.manifest_path().exists() {
            return RecentProjectValidation::Missing;
        }
        if ProjectManifest::load(paths.manifest_path()).is_err() {
            return RecentProjectValidation::InvalidManifest;
        }
        match EditorProjectDocument::load_from_path(&root) {
            Ok(_) => RecentProjectValidation::Valid,
            Err(_) => RecentProjectValidation::InvalidProject,
        }
    }

    pub fn show_welcome_page(&self) -> Result<(), EditorError> {
        let mut registry = self.view_registry.lock().unwrap();
        if registry
            .descriptor(&ViewDescriptorId::new(WELCOME_DESCRIPTOR_ID))
            .is_none()
        {
            registry
                .register_view(welcome_view_descriptor())
                .map_err(EditorError::Registry)?;
        }
        let instance = if let Some(existing) = registry
            .instance(&ViewInstanceId::new(WELCOME_INSTANCE_ID))
            .cloned()
        {
            existing
        } else {
            registry
                .restore_instance(welcome_view_instance())
                .map_err(EditorError::Registry)?
        };
        drop(registry);

        self.attach_instance(
            instance,
            ViewHost::ExclusivePage(MainPageId::new(WELCOME_PAGE_ID)),
        )?;
        self.apply_layout_command(LayoutCommand::ActivateMainPage {
            page_id: MainPageId::new(WELCOME_PAGE_ID),
        })?;
        Ok(())
    }

    pub fn dismiss_welcome_page(&self) -> Result<(), EditorError> {
        let _ = self.close_view(&ViewInstanceId::new(WELCOME_INSTANCE_ID));
        let _ = self.apply_layout_command(LayoutCommand::ActivateMainPage {
            page_id: MainPageId::workbench(),
        })?;
        Ok(())
    }
}

fn canonical_project_root(path: &Path) -> Result<PathBuf, std::io::Error> {
    let root = project_root_path(path)
        .map_err(|error| std::io::Error::new(std::io::ErrorKind::InvalidInput, error))?;
    if root.exists() {
        fs::canonicalize(root)
    } else {
        Ok(root)
    }
}

pub(super) fn welcome_view_descriptor() -> ViewDescriptor {
    ViewDescriptor::new(
        ViewDescriptorId::new(WELCOME_DESCRIPTOR_ID),
        ViewKind::ActivityWindow,
        "Welcome",
    )
    .with_preferred_host(PreferredHost::ExclusiveMainPage)
    .with_default_constraints(default_constraints_for_content(ViewContentKind::Welcome))
    .with_icon_key("welcome")
}

fn welcome_view_instance() -> ViewInstance {
    ViewInstance {
        instance_id: ViewInstanceId::new(WELCOME_INSTANCE_ID),
        descriptor_id: ViewDescriptorId::new(WELCOME_DESCRIPTOR_ID),
        title: "Welcome".to_string(),
        serializable_payload: Value::Null,
        dirty: false,
        host: ViewHost::ExclusivePage(MainPageId::new(WELCOME_PAGE_ID)),
    }
}
