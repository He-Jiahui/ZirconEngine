use std::path::{Path, PathBuf};
use std::sync::Arc;

use zircon_runtime::core::framework::project::ProjectPluginManifest;
use zircon_runtime_interface::project::{ProjectLaunchIntent, RenderedProjectTemplate};

use super::{ProjectPreflightCompositionPlan, ProjectPreflightReceipt};

/// Data-only launch evidence produced before product module composition.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ProjectLaunchPreflight {
    intent: ProjectLaunchIntent,
    target: ProjectLaunchPreflightTarget,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum ProjectLaunchPreflightTarget {
    Existing(ProjectPreflightReceipt),
    Create {
        target_root: PathBuf,
        rendered: Arc<RenderedProjectTemplate>,
        composition: ProjectPreflightCompositionPlan,
    },
}

impl ProjectLaunchPreflight {
    pub(super) fn from_target(
        intent: ProjectLaunchIntent,
        target: ProjectLaunchPreflightTarget,
    ) -> Self {
        Self { intent, target }
    }

    pub fn intent(&self) -> &ProjectLaunchIntent {
        &self.intent
    }

    pub fn target_root(&self) -> &Path {
        match &self.target {
            ProjectLaunchPreflightTarget::Existing(receipt) => receipt.root(),
            ProjectLaunchPreflightTarget::Create { target_root, .. } => target_root,
        }
    }

    pub fn approved_project_plugins(&self) -> &ProjectPluginManifest {
        match &self.target {
            ProjectLaunchPreflightTarget::Existing(receipt) => {
                receipt.composition().approved_project_plugins()
            }
            ProjectLaunchPreflightTarget::Create { composition, .. } => {
                composition.approved_project_plugins()
            }
        }
    }

    pub(crate) fn into_parts(self) -> (ProjectLaunchIntent, ProjectLaunchPreflightTarget) {
        (self.intent, self.target)
    }
}
