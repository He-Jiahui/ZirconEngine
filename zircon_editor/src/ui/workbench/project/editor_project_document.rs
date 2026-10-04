use std::path::PathBuf;

use zircon_runtime::asset::{project::ProjectManifest, ProjectInfo};
use zircon_runtime::scene::Scene;

use crate::core::settings::SettingsProjectLayerLoad;

use super::project_editor_workspace::ProjectEditorWorkspace;

#[derive(Clone, Debug, PartialEq)]
/// runtime项目激活后的启动材料；world尚需生命周期提交，workspace回退和settings来源必须保留诊断。
pub struct EditorProjectDocument {
    pub root_path: PathBuf,
    pub manifest: ProjectManifest,
    pub project_info: ProjectInfo,
    pub project_settings: ProjectSettingsLoadState,
    pub world: Scene,
    pub editor_workspace: Option<ProjectEditorWorkspace>,
    pub workspace_restore_diagnostics: Vec<EditorWorkspaceRestoreDiagnostic>,
}

/// Startup provenance for the project-scoped settings file.
///
/// Missing or invalid settings remain explicit so product startup cannot describe a fallback-only
/// activation as if it had loaded persisted project settings.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ProjectSettingsLoadState {
    Persisted { path: PathBuf, schema_version: u32 },
    Missing { path: PathBuf },
    Invalid { path: PathBuf, message: String },
}

impl ProjectSettingsLoadState {
    pub fn startup_status(&self) -> &'static str {
        match self {
            Self::Persisted {
                schema_version: 1, ..
            } => "persisted-v1",
            Self::Persisted { .. } => "persisted",
            Self::Missing { .. } => "degraded-missing",
            Self::Invalid { .. } => "degraded-invalid",
        }
    }

    pub(crate) fn from_authority_load(load: SettingsProjectLayerLoad) -> Self {
        match load {
            SettingsProjectLayerLoad::Persisted {
                path,
                schema_version,
            } => Self::Persisted {
                path,
                schema_version,
            },
            SettingsProjectLayerLoad::Missing { path } => Self::Missing { path },
            SettingsProjectLayerLoad::Invalid { path, message } => Self::Invalid { path, message },
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
/// 辅助workspace读取或解码失败的显式说明；允许用默认布局继续，不应静默视作恢复成功。
pub struct EditorWorkspaceRestoreDiagnostic {
    pub path: PathBuf,
    pub message: String,
}

impl EditorWorkspaceRestoreDiagnostic {
    pub(in crate::ui::workbench::project) fn new(
        path: impl Into<PathBuf>,
        message: impl Into<String>,
    ) -> Self {
        Self {
            path: path.into(),
            message: message.into(),
        }
    }
}
