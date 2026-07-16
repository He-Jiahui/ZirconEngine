use std::sync::Arc;

#[cfg(test)]
use crate::asset::ProjectAssetManager;
use crate::asset::ProjectAssetManagerAccess;
use crate::core::framework::render::{GeometrySourceDescriptor, ShadingModelDescriptor};
use crate::graphics::{
    RenderFeatureDescriptor, RenderPassExecutorRegistration, RuntimePrepareCollectorRegistration,
};

use crate::graphics::types::GraphicsError;

use super::super::super::overlay::EmptyViewportIconSource;
use super::super::scene_renderer::SceneRenderer;

impl SceneRenderer {
    #[cfg(test)]
    pub(crate) fn new_for_test(
        asset_manager: Arc<ProjectAssetManager>,
    ) -> Result<Self, GraphicsError> {
        Self::new(ProjectAssetManagerAccess::for_test(asset_manager))
    }

    pub fn new(asset_manager: ProjectAssetManagerAccess) -> Result<Self, GraphicsError> {
        Self::new_with_icon_source(asset_manager, Arc::new(EmptyViewportIconSource))
    }

    pub fn new_with_plugin_render_features(
        asset_manager: ProjectAssetManagerAccess,
        render_features: impl IntoIterator<Item = RenderFeatureDescriptor>,
        render_pass_executors: impl IntoIterator<Item = RenderPassExecutorRegistration>,
    ) -> Result<Self, GraphicsError> {
        Self::new_with_plugin_render_extensions(
            asset_manager,
            render_features,
            render_pass_executors,
            Vec::new(),
        )
    }

    #[cfg(test)]
    pub(crate) fn new_for_test_with_plugin_render_features(
        asset_manager: Arc<ProjectAssetManager>,
        render_features: impl IntoIterator<Item = RenderFeatureDescriptor>,
        render_pass_executors: impl IntoIterator<Item = RenderPassExecutorRegistration>,
    ) -> Result<Self, GraphicsError> {
        Self::new_with_plugin_render_features(
            ProjectAssetManagerAccess::for_test(asset_manager),
            render_features,
            render_pass_executors,
        )
    }

    pub fn new_with_plugin_render_extensions(
        asset_manager: ProjectAssetManagerAccess,
        render_features: impl IntoIterator<Item = RenderFeatureDescriptor>,
        render_pass_executors: impl IntoIterator<Item = RenderPassExecutorRegistration>,
        runtime_prepare_collectors: impl IntoIterator<Item = RuntimePrepareCollectorRegistration>,
    ) -> Result<Self, GraphicsError> {
        Self::new_with_plugin_render_extensions_and_shading_models(
            asset_manager,
            render_features,
            render_pass_executors,
            runtime_prepare_collectors,
            Vec::new(),
            Vec::new(),
        )
    }

    pub fn new_with_plugin_render_extensions_and_shading_models(
        asset_manager: ProjectAssetManagerAccess,
        render_features: impl IntoIterator<Item = RenderFeatureDescriptor>,
        render_pass_executors: impl IntoIterator<Item = RenderPassExecutorRegistration>,
        runtime_prepare_collectors: impl IntoIterator<Item = RuntimePrepareCollectorRegistration>,
        plugin_geometry_sources: impl IntoIterator<Item = GeometrySourceDescriptor>,
        plugin_shading_models: impl IntoIterator<Item = ShadingModelDescriptor>,
    ) -> Result<Self, GraphicsError> {
        Self::new_with_icon_source_and_plugin_render_features_and_shading_models(
            asset_manager,
            Arc::new(EmptyViewportIconSource),
            render_features,
            render_pass_executors,
            runtime_prepare_collectors,
            plugin_geometry_sources,
            plugin_shading_models,
        )
    }

    #[cfg(test)]
    pub(crate) fn new_for_test_with_plugin_render_extensions_and_shading_models(
        asset_manager: Arc<ProjectAssetManager>,
        render_features: impl IntoIterator<Item = RenderFeatureDescriptor>,
        render_pass_executors: impl IntoIterator<Item = RenderPassExecutorRegistration>,
        runtime_prepare_collectors: impl IntoIterator<Item = RuntimePrepareCollectorRegistration>,
        plugin_geometry_sources: impl IntoIterator<Item = GeometrySourceDescriptor>,
        plugin_shading_models: impl IntoIterator<Item = ShadingModelDescriptor>,
    ) -> Result<Self, GraphicsError> {
        Self::new_with_plugin_render_extensions_and_shading_models(
            ProjectAssetManagerAccess::for_test(asset_manager),
            render_features,
            render_pass_executors,
            runtime_prepare_collectors,
            plugin_geometry_sources,
            plugin_shading_models,
        )
    }
}
