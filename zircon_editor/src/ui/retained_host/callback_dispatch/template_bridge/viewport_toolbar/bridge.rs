use std::{collections::BTreeMap, sync::Arc};

use zircon_runtime_interface::ui::{
    binding::UiEventKind,
    layout::{UiFrame, UiSize},
    surface::UiSurfaceFrame,
};

use crate::ui::binding::EditorUiBinding;
use crate::ui::retained_host::callback_dispatch::constants::BUILTIN_VIEWPORT_TOOLBAR_DOCUMENT_ID;
use crate::ui::template_runtime::{
    EditorUiHostRuntime, RetainedUiHostProjection, RetainedUiProjection,
};

#[cfg(test)]
use super::super::projection_support::load_builtin_runtime;
use super::super::projection_support::project_builtin_document_with_runtime;
use super::error::BuiltinViewportToolbarTemplateBridgeError;
use super::host_projection::{
    build_builtin_viewport_toolbar_surface, project_builtin_viewport_toolbar_host_projection,
    rebuild_builtin_viewport_toolbar_surface,
};
use super::surface_frame_cache::ViewportToolbarSurfaceFrameCache;

pub(crate) struct BuiltinViewportToolbarTemplateBridge {
    runtime: Arc<EditorUiHostRuntime>,
    projection: RetainedUiProjection,
    bindings_by_control: BTreeMap<String, BTreeMap<UiEventKind, EditorUiBinding>>,
    surface: zircon_runtime::ui::surface::UiSurface,
    host_projection: RetainedUiHostProjection,
    surface_frame_cache: ViewportToolbarSurfaceFrameCache,
    #[cfg(test)]
    layout_recompute_count: usize,
}

impl BuiltinViewportToolbarTemplateBridge {
    #[cfg(test)]
    pub(crate) fn new() -> Result<Self, BuiltinViewportToolbarTemplateBridgeError> {
        let runtime = Arc::new(load_builtin_runtime()?);
        Self::new_with_runtime(runtime)
    }

    pub(crate) fn new_with_runtime(
        runtime: Arc<EditorUiHostRuntime>,
    ) -> Result<Self, BuiltinViewportToolbarTemplateBridgeError> {
        let projection =
            project_builtin_document_with_runtime(&runtime, BUILTIN_VIEWPORT_TOOLBAR_DOCUMENT_ID)?;
        let mut bindings_by_control =
            BTreeMap::<String, BTreeMap<UiEventKind, EditorUiBinding>>::new();
        for projected_binding in &projection.bindings {
            let path = projected_binding.binding.path();
            bindings_by_control
                .entry(path.control_id.clone())
                .or_default()
                .insert(path.event_kind, projected_binding.binding.clone());
        }
        let surface =
            build_builtin_viewport_toolbar_surface(runtime.as_ref(), UiSize::new(1280.0, 28.0))?;
        let host_projection = project_builtin_viewport_toolbar_host_projection(
            runtime.as_ref(),
            &projection,
            &surface,
        )?;
        Ok(Self {
            runtime,
            projection,
            bindings_by_control,
            surface,
            host_projection,
            surface_frame_cache: ViewportToolbarSurfaceFrameCache::default(),
            #[cfg(test)]
            layout_recompute_count: 0,
        })
    }

    pub(crate) fn recompute_layout(
        &mut self,
        surface_size: UiSize,
    ) -> Result<(), BuiltinViewportToolbarTemplateBridgeError> {
        rebuild_builtin_viewport_toolbar_surface(&mut self.surface, surface_size)?;
        self.host_projection = project_builtin_viewport_toolbar_host_projection(
            self.runtime.as_ref(),
            &self.projection,
            &self.surface,
        )?;
        #[cfg(test)]
        {
            self.layout_recompute_count = self.layout_recompute_count.saturating_add(1);
        }
        Ok(())
    }

    pub(crate) fn binding_for_control(
        &self,
        control_id: &str,
        event_kind: UiEventKind,
    ) -> Option<&EditorUiBinding> {
        self.bindings_by_control
            .get(control_id)
            .and_then(|bindings| bindings.get(&event_kind))
    }

    #[cfg(test)]
    pub(crate) fn layout_recompute_count(&self) -> usize {
        self.layout_recompute_count
    }

    pub(crate) fn control_frame_for_control(&self, control_id: &str) -> Option<UiFrame> {
        self.host_projection
            .node_by_control_id(control_id)
            .map(|node| node.frame)
    }

    pub(crate) fn surface_frame_for_projection_controls<F>(
        &mut self,
        surface_key: &str,
        surface_size: UiSize,
        hit_control_id: F,
    ) -> Arc<UiSurfaceFrame>
    where
        F: FnMut(&str) -> Option<String>,
    {
        self.surface_frame_cache.resolve(
            &self.host_projection,
            surface_key,
            surface_size,
            None,
            hit_control_id,
        )
    }

    pub(crate) fn surface_frame_for_projection_controls_with_hit_route_key<F>(
        &mut self,
        surface_key: &str,
        surface_size: UiSize,
        hit_route_key: &[&str],
        hit_control_id: F,
    ) -> Arc<UiSurfaceFrame>
    where
        F: FnMut(&str) -> Option<String>,
    {
        self.surface_frame_cache.resolve(
            &self.host_projection,
            surface_key,
            surface_size,
            Some(hit_route_key),
            hit_control_id,
        )
    }

    pub(crate) fn surface_frame_from_cached_layout_for_projection_controls<F>(
        &mut self,
        surface_key: &str,
        surface_size: UiSize,
        hit_control_id: F,
    ) -> Option<Arc<UiSurfaceFrame>>
    where
        F: FnMut(&str) -> Option<String>,
    {
        self.surface_frame_cache.resolve_if_layout_matches(
            surface_key,
            surface_size,
            None,
            hit_control_id,
        )
    }

    pub(crate) fn surface_frame_from_cached_layout_for_projection_controls_with_hit_route_key<F>(
        &mut self,
        surface_key: &str,
        surface_size: UiSize,
        hit_route_key: &[&str],
        hit_control_id: F,
    ) -> Option<Arc<UiSurfaceFrame>>
    where
        F: FnMut(&str) -> Option<String>,
    {
        self.surface_frame_cache.resolve_if_layout_matches(
            surface_key,
            surface_size,
            Some(hit_route_key),
            hit_control_id,
        )
    }

    #[cfg(test)]
    pub(crate) fn hit_control_projection_count(&self) -> usize {
        self.surface_frame_cache.hit_control_projection_count()
    }
}
