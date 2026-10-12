use std::sync::Arc;

use super::super::{
    RenderAssetGpuUploadLimits, RenderAssetResidencyManager, RenderAssetResidencyTicket,
};
use super::{
    RenderAssetSemanticExecutor, RenderAssetSemanticExecutorAdmissionError,
    RenderAssetSemanticExecutorCloseReport, RenderAssetSemanticExecutorDiagnostics,
    RenderAssetSemanticExecutorInitError, RenderAssetSemanticExecutorLimits,
    RenderAssetSemanticExecutorMaintenanceBudget, RenderAssetSemanticExecutorMaintenanceError,
    RenderAssetSemanticExecutorMaintenanceReport,
};
use crate::asset::artifact::{
    RenderArtifactBlockLoader, RenderArtifactBlockLoaderInitError, RenderArtifactBlockLoaderLimits,
    RenderArtifactManifestLoader, RenderArtifactManifestLoaderInitError,
    RenderArtifactManifestLoaderLimits, RenderArtifactStore,
};
use crate::asset::pipeline::manager::{ProjectAssetGenerationToken, ProjectGenerationMatch};
use crate::asset::ProjectAssetManagerAccess;
use crate::core::CoreError;

/// All bounded loader and prepare limits for one project-generation executor.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct RenderAssetSemanticExecutorOwnerConfig {
    manifest_loader: RenderArtifactManifestLoaderLimits,
    block_loader: RenderArtifactBlockLoaderLimits,
    max_in_flight: usize,
    upload: RenderAssetGpuUploadLimits,
    target_platform: Arc<str>,
}

impl RenderAssetSemanticExecutorOwnerConfig {
    pub(crate) fn new(
        manifest_loader: RenderArtifactManifestLoaderLimits,
        block_loader: RenderArtifactBlockLoaderLimits,
        max_in_flight: usize,
        upload: RenderAssetGpuUploadLimits,
        target_platform: Arc<str>,
    ) -> Self {
        Self {
            manifest_loader,
            block_loader,
            max_in_flight,
            upload,
            target_platform,
        }
    }

    fn executor_limits(&self) -> RenderAssetSemanticExecutorLimits {
        RenderAssetSemanticExecutorLimits::new(self.max_in_flight, self.upload)
    }
}

#[derive(Debug, thiserror::Error)]
pub(crate) enum RenderAssetSemanticExecutorOwnerInitError {
    #[error("project asset manager has no active project generation")]
    NoActiveProject,
    #[error(transparent)]
    AssetManager(#[from] CoreError),
    #[error(transparent)]
    ManifestLoader(#[from] RenderArtifactManifestLoaderInitError),
    #[error(transparent)]
    BlockLoader(#[from] RenderArtifactBlockLoaderInitError),
    #[error(transparent)]
    Executor(#[from] RenderAssetSemanticExecutorInitError),
}

#[derive(Debug, thiserror::Error)]
pub(crate) enum RenderAssetSemanticExecutorOwnerAdmissionError {
    #[error("project generation was superseded before semantic residency admission")]
    Superseded {
        newer_same_project_generation: bool,
        close: RenderAssetSemanticExecutorCloseReport,
    },
    #[error(transparent)]
    AssetManager(#[from] CoreError),
    #[error(transparent)]
    Executor(#[from] RenderAssetSemanticExecutorAdmissionError),
}

#[derive(Debug, thiserror::Error)]
pub(crate) enum RenderAssetSemanticExecutorOwnerMaintenanceError {
    #[error("project generation was superseded before semantic residency maintenance")]
    Superseded {
        newer_same_project_generation: bool,
        close: RenderAssetSemanticExecutorCloseReport,
    },
    #[error(transparent)]
    AssetManager(#[from] CoreError),
    #[error(transparent)]
    Executor(#[from] RenderAssetSemanticExecutorMaintenanceError),
}

/// Owns the artifact store, asynchronous loaders, and semantic executor for one project
/// generation. It is intentionally not a WGPU owner: neutral upload plans leave this boundary and
/// are submitted by the unique RHI device owner.
pub(crate) struct RenderAssetSemanticExecutorOwner {
    project: ProjectAssetManagerAccess,
    generation: ProjectAssetGenerationToken,
    executor: RenderAssetSemanticExecutor,
}

impl RenderAssetSemanticExecutorOwner {
    pub(crate) fn try_new(
        asset_manager: &ProjectAssetManagerAccess,
        config: &RenderAssetSemanticExecutorOwnerConfig,
    ) -> Result<Self, RenderAssetSemanticExecutorOwnerInitError> {
        let (core, project) = asset_manager.resolve_with_core()?;
        let snapshot = project
            .current_project_generation_snapshot()
            .ok_or(RenderAssetSemanticExecutorOwnerInitError::NoActiveProject)?;
        let (project_snapshot, generation) = snapshot.into_parts();
        let store =
            RenderArtifactStore::new(project_snapshot.paths().asset_artifact_root().to_path_buf());
        let runtime = core.task_graph();
        let manifest_loader =
            RenderArtifactManifestLoader::new(store.clone(), config.manifest_loader, runtime)?;
        let block_loader = RenderArtifactBlockLoader::new(store, config.block_loader, runtime)?;
        let executor = RenderAssetSemanticExecutor::try_new(
            manifest_loader,
            block_loader,
            Arc::clone(&config.target_platform),
            config.executor_limits(),
            runtime,
        )?;
        Ok(Self {
            project: asset_manager.clone(),
            generation,
            executor,
        })
    }

    pub(crate) fn generation(&self) -> &ProjectAssetGenerationToken {
        &self.generation
    }

    pub(crate) fn diagnostics(&self) -> RenderAssetSemanticExecutorDiagnostics {
        self.executor.diagnostics()
    }

    pub(crate) fn admit(
        &mut self,
        ticket: RenderAssetResidencyTicket,
        priority: crate::asset::artifact::RenderArtifactIoPriority,
        deadline: Option<std::time::Instant>,
        residency: &mut RenderAssetResidencyManager,
    ) -> Result<(), RenderAssetSemanticExecutorOwnerAdmissionError> {
        self.ensure_current_admission()?;
        self.executor
            .admit(ticket, priority, deadline, residency)
            .map_err(Into::into)
    }

    pub(crate) fn maintain(
        &mut self,
        budget: RenderAssetSemanticExecutorMaintenanceBudget,
        residency: &mut RenderAssetResidencyManager,
    ) -> Result<
        RenderAssetSemanticExecutorMaintenanceReport,
        RenderAssetSemanticExecutorOwnerMaintenanceError,
    > {
        self.ensure_current_maintenance()?;
        self.executor
            .maintain(budget, residency)
            .map_err(Into::into)
    }

    // TODO: [CR-R02-runtime_wave12_graphics_resource_residency-0005] 待确认中性计划的唯一提交与绑定 owner；当前只找到提取测试及路由定义，缺少实际帧交接证据，需追踪提交、ticket 绑定和失败回收契约。
    pub(crate) fn take_next_ready_upload(
        &mut self,
    ) -> Option<super::super::RenderAssetGpuUploadPlan> {
        self.executor.take_next_ready_upload()
    }

    pub(crate) fn take_next_failure(
        &mut self,
    ) -> Option<super::super::RenderAssetSemanticExecutorFailure> {
        self.executor.take_next_failure()
    }

    pub(crate) fn cancel(&mut self, ticket: &RenderAssetResidencyTicket) -> bool {
        self.executor.cancel(ticket)
    }

    pub(crate) fn close(&mut self) -> RenderAssetSemanticExecutorCloseReport {
        self.executor.close()
    }

    fn ensure_current_admission(
        &mut self,
    ) -> Result<(), RenderAssetSemanticExecutorOwnerAdmissionError> {
        let project = match self.project.resolve() {
            Ok(project) => project,
            Err(error) => {
                self.executor.close();
                return Err(error.into());
            }
        };
        match project.check_project_generation(&self.generation) {
            ProjectGenerationMatch::Current => Ok(()),
            ProjectGenerationMatch::Superseded {
                newer_same_project_generation,
            } => {
                let close = self.executor.close();
                Err(RenderAssetSemanticExecutorOwnerAdmissionError::Superseded {
                    newer_same_project_generation,
                    close,
                })
            }
        }
    }

    fn ensure_current_maintenance(
        &mut self,
    ) -> Result<(), RenderAssetSemanticExecutorOwnerMaintenanceError> {
        let project = match self.project.resolve() {
            Ok(project) => project,
            Err(error) => {
                self.executor.close();
                return Err(error.into());
            }
        };
        match project.check_project_generation(&self.generation) {
            ProjectGenerationMatch::Current => Ok(()),
            ProjectGenerationMatch::Superseded {
                newer_same_project_generation,
            } => {
                let close = self.executor.close();
                Err(
                    RenderAssetSemanticExecutorOwnerMaintenanceError::Superseded {
                        newer_same_project_generation,
                        close,
                    },
                )
            }
        }
    }
}

impl Drop for RenderAssetSemanticExecutorOwner {
    fn drop(&mut self) {
        self.executor.close();
    }
}
